use std::vec;

use assembler_errors::AssemblerError;
use program_parser::{program, Program};
use symbols::{Symbol, SymbolTable, SymbolType};

use crate::instruction::Opcode;

pub mod assembler_errors;
pub mod directive_parsers;
pub mod instruction_parsers;
pub mod label_parsers;
pub mod opcode_parsers;
pub mod operand_parser;
pub mod program_parser;
pub mod register_parser;
pub mod symbols;

/// Magic number that begins every bytecode file prefix. These spell out EPIE in ASCII, if you were wondering.
pub const PIE_HEADER_PREFIX: [u8; 4] = [45, 50, 49, 45];

/// Constant that determines how long the header is. There are 60 zeros left after the prefix, for later usage if needed.
pub const PIE_HEADER_LENGTH: usize = 64;

pub fn prepend_header(mut bytes: Vec<u8>) -> Vec<u8> {
    let mut prepension = vec![];
    for byte in PIE_HEADER_PREFIX.into_iter() {
        prepension.push(byte);
    }
    while prepension.len() < PIE_HEADER_LENGTH {
        prepension.push(0);
    }
    prepension.append(&mut bytes);
    prepension
}

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Op { code: Opcode },
    Register { reg_num: u8 },
    IntegerOperand { value: i32 },
    LabelDeclaration { name: String },
    LabelUsage { name: String },
    Directive { name: String },
    IrString { name: String },
    Comment,
}

#[derive(Debug, Default)]
pub struct Assembler {
    /// Tracks which phase the assember is in
    phase: AssemblerPhase,
    /// Symbol table for constants and variables
    pub symbols: SymbolTable,
    /// The read-only data section constants are put in
    pub ro: Vec<u8>,
    /// The compiled bytecode generated from the assembly instructions
    pub bytecode: Vec<u8>,
    /// Tracks the current offset of the read-only section
    ro_offset: u32,
    /// A list of all the sections we've seen in the code
    sections: Vec<AssemblerSection>,
    /// The current section the assembler is in
    current_section: Option<AssemblerSection>,
    /// The current instruction the assembler is converting to bytecode
    current_instruction: u32,
    /// Any errors we find along the way. At the end, we'll present them to the user.
    errors: Vec<AssemblerError>,
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler {
            current_instruction: 0,
            ro_offset: 0,
            ro: vec![],
            bytecode: vec![],
            sections: vec![],
            errors: vec![],
            phase: AssemblerPhase::First,
            symbols: SymbolTable::new(),
            current_section: None,
        }
    }

    pub fn assemble(&mut self, raw: &str) -> Option<Vec<u8>> {
        match program(raw) {
            Ok((_reminder, program)) => {
                //First get the header so we can smush it into the bytecode letter
                let mut assembled_program = self.write_pie_header();
                self.process_first_phase(&program);
                let mut body = self.process_second_phase(&program);

                // Merge the header with the populated body vector
                assembled_program.append(&mut body);
                Some(assembled_program)
            },
            Err(e) => {
                eprintln!("There was an error assembling the code: {:?}", e);
                None
            },
        }
    }

    fn write_pie_header(&self) -> Vec<u8> {
        // let mut header = vec![];
        // for byte in PIE_HEADER_PREFIX.into_iter() {
        //     header.push(byte);
        // }

        // while header.len() <= PIE_HEADER_LENGTH {
        //     header.push(0 as u8);
        // }
        // header
        prepend_header(vec![])
    }

    /// The first phase extracts all the labels and builds the symbol table
    fn process_first_phase(&mut self, p: &Program) {
        self.extract_labels(p);
        self.phase = AssemblerPhase::Second;
    }

    /// The second phase is then called, which just calls to_bytes on every AssemblerInstruction
    fn process_second_phase(&mut self, p: &Program) -> Vec<u8> {
        let mut program = vec![];
        for i in &p.instructions {
            let mut bs = i.to_bytes(&self.symbols);
            program.append(&mut bs);
        }
        program
    }

    fn extract_labels(&mut self, p: &Program) {
        let mut c = 0;
        for i in &p.instructions {
            if i.is_label() {
                match i.get_label_name() {
                    Some(name) => {
                        let symbol = Symbol::new_with_offset(name, SymbolType::Label, c);
                        self.symbols.add_symbol(symbol);
                    },
                    None => {},
                }
            }
            c += 4;
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssemblerPhase {
    First,
    Second,
}

impl Default for AssemblerPhase {
    fn default() -> Self {
        AssemblerPhase::First
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssemblerSection {
    Data { starting_instruction: Option<u32> },
    Code { starting_instruction: Option<u32> },
    Unknown,
}

impl Default for AssemblerSection {
    fn default() -> Self {
        AssemblerSection::Unknown
    }
}

impl From<&str> for AssemblerSection {
    fn from(value: &str) -> Self {
        match value {
            ".data" => AssemblerSection::Data {
                starting_instruction: None,
            },
            ".code" => AssemblerSection::Data {
                starting_instruction: None,
            },
            _ => AssemblerSection::Unknown,
        }
    }
}

#[cfg(test)]
#[allow(unused_variables, unused_mut)]
mod tests {
    use crate::{
        assembler::{
            directive_parsers::directive,
            program_parser::program,
            symbols::{Symbol, SymbolTable, SymbolType},
        },
        vm::VM,
    };

    use super::Assembler;

    #[test]
    fn test_symbol_table() {
        let mut sym = SymbolTable::new();
        let new_symbol = Symbol::new_with_offset("test".to_string(), SymbolType::Label, 12);
        sym.add_symbol(new_symbol);
        assert_eq!(sym.symbols.len(), 1);
        let v = sym.symbol_value("test");
        assert_eq!(true, v.is_some());
        let v = v.unwrap();
        assert_eq!(v, 12);
        let v = sym.symbol_value("does_not_exist");
        assert_eq!(v.is_some(), false);
    }

    #[test]
    /// Tests that code which does not declare a segment first does not work
    fn test_first_phase_no_segment() {
        let mut asm = Assembler::new();
        let test_string = "hello: .asciiz 'Fail'";
        let result = directive(test_string);
        assert_eq!(result.is_ok(), true);
        let (_, p) = result.unwrap();
        // asm.process_first_phase(&p);/.
        // assert_eq!(asm.errors.len(), 1);
    }

    #[test]
    /// Tests that code inside a proper segment works
    fn test_first_phase_inside_segment() {
        let mut asm = Assembler::new();
        let test_string = ".data\ntest: .asciiz 'Hello'";
        let result = program(test_string);
        assert_eq!(result.is_ok(), true);
        let (_, p) = result.unwrap();
        asm.process_first_phase(&p);
        // assert_eq!(asm.errors.len(), 0);
    }

    #[test]
    /// Tests assembly a small but correct program
    fn test_assemble_program() {
        let mut asm = Assembler::new();
        let test_string = ".data\n.code\nload $0 #100\nload $1 #1\nload $2 #0\ntest: inc $0\nneq $0 $2\njmpe @test\nhlt";
        let program = asm.assemble(test_string).unwrap();
        let mut vm = VM::new();
        assert_eq!(program.len(), 92, "\nProgram: {:?}\n", program);
        vm.add_bytes(program);
        assert_eq!(vm.program.len(), 92);
    }
}
