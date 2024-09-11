use crate::instruction::Opcode;

pub mod opcode;
pub mod opcode_parsers;
pub mod operand_parser;
pub mod register_parser;

#[derive(Debug, PartialEq)]
pub enum Token {
    Op { code: Opcode },
    Register { reg_num: u8 },
    IntegerOperand { value: i32 },
}
