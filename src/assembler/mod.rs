use std::vec;

use assembler_errors::AssemblerError;
use byteorder::{LittleEndian, WriteBytesExt};
use instruction_parsers::AssemblerInstruction;
use log::{debug, error, warn};
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
pub const PIE_HEADER_PREFIX: [u8; 4] = [45, 50, 49, 45]; // Hello

/// Constant that determines how long the header is. There are 60 zeros left after the prefix, for later usage if needed.
pub const PIE_HEADER_LENGTH: usize = 64;

pub fn prepend_header(mut append_bytes: Vec<u8>) -> Vec<u8> {
    let mut prepension = vec![];
    for byte in PIE_HEADER_PREFIX.into_iter() {
        prepension.push(byte);
    }
    while prepension.len() < PIE_HEADER_LENGTH {
        prepension.push(0 as u8);
    }
    prepension.append(&mut append_bytes);
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
    pub errors: Vec<AssemblerError>,
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

    pub fn assemble(&mut self, raw: &str) -> Result<Vec<u8>, Vec<AssemblerError>> {
        match program(raw) {
            Ok((_reminder, program)) => {
                // If there were no parsing errors, we now have a `Vec<AssemblyInstructions>` to process.
                // `remainder` _should_ be "".
                // TODO: Add a check for `remainder`, make sure it is "".
                debug_assert!(
                    _reminder.is_empty(),
                    "_reminder should be empty: {:?}",
                    _reminder
                ); // Unlike assert, debug_assert! statements are only enabled in non optimized builds by default.

                // //First get the header so we can smush it into the bytecode letter
                // let mut assembled_program = self.write_pie_header();

                // Start processing the AssembledInstructions. This is the first pass of our two-pass assembler.
                // We pass a read-only reference down to another function.
                self.process_first_phase(&program);

                // If we accumulated any errors in the first pass, return them and don't try to do the second pass
                if !&self.errors.is_empty() {
                    // TODO: Can we avoid a clone here?
                    return Err(self.errors.clone());
                }

                // Make sure that we have at least one data section and one code section
                if self.sections.len() != 2 {
                    eprintln!("Did not find at least two sections.");
                    self.errors.push(AssemblerError::InsufficientSections);
                    return Err(self.errors.clone());
                }

                // Run the second pass, which translates opcodes and associated operands into the bytecode
                let mut body = self.process_second_phase(&program);

                // Get the header so we can smush it into the bytecode letter
                let mut assembled_program = self.write_pie_header();

                // Merge the header with the populated body vector
                assembled_program.append(&mut body);
                Ok(assembled_program)
            },
            Err(e) => {
                // If there were parsing errors, bad syntax, etc, this arm is run
                eprintln!("There was an error assembling the code: {:?}", e);
                Err(vec![AssemblerError::ParseError {
                    error: e.to_string(),
                }])
            },
        }
    }

    fn write_pie_header(&self) -> Vec<u8> {
        let mut header = vec![];
        for byte in PIE_HEADER_PREFIX.into_iter() {
            header.push(byte);
        }

        // Now we need to calculate the starting offset so that the VM knows where the RO section ends

        //First we declare an empty vector for byteorder to write to
        let mut wtr: Vec<u8> = vec![];

        wtr.write_u32::<LittleEndian>(self.ro.len() as u32).unwrap();

        // Append those 4 bytes to the header directly after the first four bytes
        header.append(&mut wtr);

        // Now pad the rest of the bytecode header
        while header.len() < PIE_HEADER_LENGTH {
            header.push(0 as u8);
        }
        header
    }

    /// The first phase extracts all the labels and builds the symbol table
    ///
    /// Runs the first pass of the two-pass assembling process.
    /// It looks for labels and puts them in the symbol table
    fn process_first_phase(&mut self, p: &Program) {
        // Iterate over every instruction, even though in the first phase we only care about labels and directives
        for i in &p.instructions {
            if i.is_label() {
                // TODO: Factor this out into another function? Put it in `process_label_declaration` maybe?
                if self.current_section.is_some() {
                    // If we have hit a segment header already (e.g., `.code`) then we are ok
                    self.process_label_declaration(&i);
                } else {
                    // If we have *not* hit a segment header yet, then we have a label outside of a segment, which is not allowed
                    self.errors.push(AssemblerError::NoSegmentDeclarationFound {
                        instruction: self.current_instruction,
                    });
                }
            }

            if i.is_directive() {
                self.process_directive(i);
            }
            // This is used to keep track of which instruction we hit an error on
            self.current_instruction += 1;
        }
        self.phase = AssemblerPhase::Second;
    }

    /// The second phase is then called, which just calls to_bytes on every AssemblerInstruction
    fn process_second_phase(&mut self, p: &Program) -> Vec<u8> {
        // 重新启动指令计数
        self.current_instruction = 0;
        // 我们将把要执行的字节码放在一个单独的 Vec 中，这样我们就可以做一些后处理，然后将其与头部和只读部分合并
        // 例子可以是优化，额外检查，等等
        let mut program = vec![];

        for i in &p.instructions {
            if i.is_opcode() {
                // 操作码知道如何正确地将自己转换为 32 位，所以我们可以直接调用 `to_bytes` 并追加到我们的程序中
                let mut bytes = i.to_bytes(&self.symbols);
                program.append(&mut bytes);
            }

            if i.is_directive() {
                // 在这个阶段，我们可以有指令，但我们在第一阶段关心的不同类型的指令。指令本身可以检查汇编器
                // 在哪个阶段，并决定如何处理它
                self.process_directive(i)
            }

            self.current_instruction += 1;
        }
        program
    }

    /// 处理一个标签声明，如：
    /// hello: .asciiz 'Hello'
    fn process_label_declaration(&mut self, i: &AssemblerInstruction) {
        // 检查标签是否为 None 或 String
        let name = match i.get_label_name() {
            Some(name) => name,
            None => {
                self.errors
                    .push(AssemblerError::StringConstantDeclaredWithoutLabel {
                        instruction: self.current_instruction,
                    });
                return;
            },
        };

        debug!(
            "Found label declaration: {} on line {}",
            name, self.current_instruction
        );

        // 检查标签是否已经在使用中（在符号表中有条目）
        // TODO: 有更干净的方法来做这个吗？
        if self.symbols.has_symbol(&name) {
            self.errors.push(AssemblerError::SymbolAlreadyDeclared);
            return;
        }

        // 到了这里，那它就不是我们之前见过的符号，所以把它放在表中
        let symbol = Symbol::new(name, SymbolType::Label);
        self.symbols.add_symbol(symbol);
    }

    fn process_directive(&mut self, i: &AssemblerInstruction) {
        // First let's make sure we have a parseable name
        let directive_name = match i.get_directive_name() {
            Some(name) => name,
            None => {
                error!("Directive has an invalid name: {:?}", i);
                return;
            },
        };

        // Now check if there were any operands.
        if i.has_operands() {
            // If it _does_ have operands, we need to figure out which directive it was
            match directive_name.as_str() {
                "asciiz" => {
                    self.handle_asciiz(i);
                },
                _ => {
                    self.errors.push(AssemblerError::UnknownDirectiveFound {
                        directive: directive_name.clone(),
                    });
                },
            }
        } else {
            self.process_section_header(&directive_name);
        }
    }

    /// Handles a declaration of a null-terminated string:
    /// hello: .asciiz 'Hello!'
    fn handle_asciiz(&mut self, i: &AssemblerInstruction) {
        // Being a constant declaration, this is only meaningful in the first pass
        if self.phase != AssemblerPhase::First {
            return;
        }

        match i.get_string_constant() {
            Some(s) => {
                match i.get_label_name() {
                    Some(name) => {
                        self.symbols.set_symbol_offset(&name, self.ro_offset);
                    },
                    None => {
                        // This would be someone typing:
                        // .asciiz 'Hello'
                        warn!("Found a string constant with no associated label!");
                        return;
                    },
                };
                // We'll read the string into the read-only section byte-by-byte
                for b in s.as_bytes() {
                    self.ro.push(*b);
                    self.ro_offset += 1;
                }
                // This is the null termination bit we are using to indicate a string has ended
                self.ro.push(0);
                self.ro_offset += 1;
            },

            None => {
                // This just means someone typed `.asciiz` for some reason
                warn!("String constant following an .asciiz was empty");
            },
        }
    }

    fn process_section_header(&mut self, header_name: &str) {
        let new_section = AssemblerSection::from(header_name);
        // Only specific section names are allowed
        if new_section == AssemblerSection::Unknown {
            warn!(
                "Found an section header that is unknown: {:#?}",
                header_name
            );
            return;
        }

        // TODO: Check if we really need to keep a list of all sections seen
        self.sections.push(new_section.clone());
        self.current_section = Some(new_section);
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
            "data" => AssemblerSection::Data {
                starting_instruction: None,
            },
            "code" => AssemblerSection::Code {
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

    #[test]
    /// Simple test of data that goes into the read only section
    fn test_ro_data() {
        let mut asm = Assembler::new();
        let test_string = ".data\ntest: .asciiz 'This is a test'\n.code\n";
        let program = asm.assemble(test_string);
        assert_eq!(program.is_ok(), true);
    }

    #[test]
    /// This tests that a section name that isn't `code` or `data` throws an error
    fn test_bad_ro_data() {
        let mut asm = Assembler::new();
        let test_string = ".code\ntest: .asciiz 'This is a test'\n.wrong\n";
        let program = asm.assemble(test_string);
        assert_eq!(program.is_ok(), false);
    }

    #[test]
    /// Tests that code which does not declare a segment first does not work
    fn test_first_phase_no_segment() {
        let mut asm = Assembler::new();
        let test_string = "hello: .asciiz 'Fail'";
        let result = program(test_string);
        assert_eq!(result.is_ok(), true);
        let (_, p) = result.unwrap();
        asm.process_first_phase(&p);
        assert_eq!(asm.errors.len(), 1);
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
        assert_eq!(asm.errors.len(), 0);
    }

    #[test]
    /// Simple test of data that goes into the read only section
    fn test_code_start_offset_written() {
        let mut asm = Assembler::new();
        let test_string = ".data\ntest1: .asciiz 'Hello'\n.code\nload $0 #100\nload $1 #1\nload $2 #0\ntest: inc $0\nneq $0 $2\njmpe @test\nhlt";
        let program = asm.assemble(test_string);
        assert_eq!(program.is_ok(), true);

        let program = program.unwrap();
        assert_eq!(program[4], 6);
    }
}
