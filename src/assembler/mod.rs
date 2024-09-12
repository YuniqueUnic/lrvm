use crate::instruction::Opcode;

pub mod directive_parsers;
pub mod instruction_parsers;
pub mod label_parsers;
pub mod opcode;
pub mod opcode_parsers;
pub mod operand_parser;
pub mod program_parser;
pub mod register_parser;

#[derive(Debug, PartialEq, Clone)]
pub enum Token {
    Op { code: Opcode },
    Register { reg_num: u8 },
    IntegerOperand { value: i32 },
    LabelDeclaration { name: String },
    LabelUsage { name: String },
    Directive { name: String },
}
