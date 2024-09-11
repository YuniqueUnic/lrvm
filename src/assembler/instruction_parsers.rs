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
    /// 将指令转换为字节码
    ///
    /// 本函数负责将 CPU 指令转换为字节码表示。它首先处理指令的操作码部分，
    /// 然后依次处理指令的每个操作数。如果操作码或操作数不能正确转换为字节码，
    /// 函数将打印错误信息并终止程序。
    ///
    /// 返回：
    ///     一个包含字节码的向量，表示该 CPU 指令
    pub fn to_bytes(&self) -> Vec<u8> {
        // 初始化存储字节码的向量
        let mut results = vec![];

        // 根据操作码将其转换为字节码
        match self.opcode {
            Token::Op { code } => {
                // 将操作码转换为 u8 类型并添加到结果向量中
                results.push(code as u8);
            },
            _ => {
                // 如果操作码字段中没有操作码，打印信息并终止程序
                println!("Non-Opcode found in opcode field");
                std::process::exit(1);
            },
        }

        // 遍历指令的操作数，将它们转换为字节码
        for operand in vec![&self.operand1, &self.operand2, &self.operand3] {
            match operand {
                Some(t) => {
                    // 如果操作数存在，调用提取函数将其添加到结果向量中
                    AssemblerInstruction::extract_operand(t, &mut results);
                },
                None => {
                    // 如果操作数不存在，跳过
                },
            }
        }

        // 返回包含指令字节码的向量
        results
    }

    /// 从解析令牌中提取操作数并将其转换为字节后存储到结果向量中。
    ///
    /// 该函数根据传入的令牌类型执行不同的操作以提取操作数。
    /// - 对于寄存器类型的令牌，它将寄存器编号作为单个字节提取。
    /// - 对于整数操作数类型的令牌，它将操作数值转换为两个字节后提取。
    /// - 对于其他类型的令牌，它打印错误信息并退出程序。
    ///
    /// 参数：
    /// - t: 指向包含操作数信息的令牌的引用。
    /// - results: 操作数提取后将字节数据推入此向量。
    fn extract_operand(t: &Token, results: &mut Vec<u8>) {
        match t {
            // 对于寄存器类型的令牌，提取并存储寄存器编号。
            Token::Register { reg_num } => {
                results.push(*reg_num);
            },
            // 对于整数操作数类型的令牌，将其值转换为两个字节后提取并存储。
            Token::IntegerOperand { value } => {
                let converted = *value as u16;
                let byte1 = converted;
                let byte2 = converted >> 8;
                // 利用大端序规则，将最高有效字节首先存储。
                // obuse the big endian rule that store the most significant byte first
                results.push(byte2 as u8);
                results.push(byte1 as u8);
            },
            // 对于其他所有令牌类型，打印错误信息并退出程序。
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
