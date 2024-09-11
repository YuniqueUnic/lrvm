use std::clone;

use nom::{
    branch::alt,
    character::complete::{line_ending, multispace0},
    combinator::{eof, map},
    error::context,
    sequence::{terminated, tuple},
    IResult,
};

use super::{
    opcode_parsers::opcode_load, operand_parser::integer_operand, register_parser::register, Token,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct AssemblerInstruction {
    opcode: Token,
    operand1: Option<Token>,
    operand2: Option<Token>,
    operand3: Option<Token>,
}

impl AssemblerInstruction {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut results = vec![];
        match self.opcode {
            Token::Op { code } => match code {
                _ => {
                    results.push(code as u8);
                },
            },
            _ => {
                println!("Non-Opcode found in opcode field");
                std::process::exit(1);
            },
        };

        for operand in vec![&self.operand1, &self.operand2, &self.operand3] {
            match operand {
                Some(t) => AssemblerInstruction::extract_operand(t, &mut results),
                None => {},
            }
        }

        results
    }

    fn extract_operand(t: &Token, results: &mut Vec<u8>) {
        match t {
            Token::Register { reg_num } => {
                results.push(*reg_num);
            },
            Token::IntegerOperand { value } => {
                let converted = *value as u16;
                let byte1 = converted;
                let byte2 = converted >> 8;
                // obuse the big endian rule that store the most significant byte first
                results.push(byte2 as u8);
                results.push(byte1 as u8);
            },
            _ => {
                println!("Opcode found in operand field");
                std::process::exit(1);
            },
        }
    }
}

pub fn instruction_one(input: &str) -> IResult<&str, AssemblerInstruction> {
    context(
        // use context to show better error msg when failed to parse
        "instruction_one",
        terminated(
            map(
                tuple((opcode_load, register, integer_operand)),
                |(o, r, i)| AssemblerInstruction {
                    opcode: o,
                    operand1: Some(r),
                    operand2: Some(i),
                    operand3: None,
                },
            ),
            alt((multispace0, line_ending, eof)),
        ),
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::{
        assembler::{instruction_parsers::AssemblerInstruction, Token},
        instruction::Opcode,
    };

    use super::instruction_one;

    #[test]
    fn test_parse_instruction_form_one() {
        let expect = AssemblerInstruction {
            opcode: Token::Op { code: Opcode::LOAD },
            operand1: Some(Token::Register { reg_num: 0 }),
            operand2: Some(Token::IntegerOperand { value: 100 }),
            operand3: None,
        };

        let result = instruction_one("load $0 #100\n");
        assert_eq!(result, Ok(("", expect.clone())));

        let result = instruction_one("load $0 #100     \n    ");
        assert_eq!(result, Ok(("", expect)));
    }
}
