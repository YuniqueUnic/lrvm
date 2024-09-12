use crate::assembler::instruction_parsers::{instruction_combined, AssemblerInstruction};
use nom::{combinator::map, error::context, multi::many1, IResult};

#[derive(Debug, PartialEq)]
pub struct Program {
    instructions: Vec<AssemblerInstruction>,
}

impl Program {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut program_bytes = vec![];
        for instruction in &self.instructions {
            program_bytes.append(&mut instruction.to_bytes());
        }
        program_bytes
    }
}

/// 解析程序的主函数
///
/// # 参数
/// - `input`: 一个字符串切片，表示程序的输入
///
/// # 返回值
/// 返回一个 `IResult<&str, Program>` 类型，其中包含两个主要部分：
/// - 第一个元素是剩余未解析的输入字符串切片
/// - 第二个元素是解析出的程序结构 `Program`
///
/// # 说明
/// 该函数使用 `context` 和 `map` 组合来解析输入字符串，将其转换为 `Program` 结构体。
/// `context` 提供了一个错误上下文，当解析发生错误时提供更多的错误信息。
/// `map` 函数将 `many1(instruction_one)` 的结果（一系列的指令）转换为一个 `Program` 实例。
/// `instruction_one` 是一个解析单个指令的函数，`many1` 保证了至少解析一个指令。
pub fn program(input: &str) -> IResult<&str, Program> {
    context(
        "program",
        map(many1(instruction_combined), |instructions| Program {
            instructions,
        }),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::program;

    #[test]
    fn test_parse_program() {
        let result = program("load $0 #100\n");
        assert_eq!(result.is_ok(), true);
        let (leftover, p) = result.unwrap();
        assert_eq!(leftover, "");
        assert_eq!(1, p.instructions.len());

        let result = program("load $0 #100   \n   ");
        assert_eq!(result.is_ok(), true);
        let (leftover, p) = result.unwrap();
        assert_eq!(leftover, "");
        assert_eq!(1, p.instructions.len());
    }

    #[test]
    fn test_program_to_bytes() {
        let result = program("load $0 #100\n");
        assert_eq!(result.is_ok(), true);
        let (_, program_res) = result.unwrap();
        let bytecode = program_res.to_bytes();
        assert_eq!(bytecode.len(), 4);
        println!("load $0 #100  ==To_Bytes==> {:?}", bytecode);

        let result = program("load $0 #1000  \n   ");
        assert_eq!(result.is_ok(), true);
        let (_, program_res) = result.unwrap();
        let bytecode = program_res.to_bytes();
        assert_eq!(bytecode.len(), 4);
        println!("load $0 #1000 ==To_Bytes==> {:?}", bytecode);
    }
}
