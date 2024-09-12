use crate::assembler::instruction_parsers::AssemblerInstruction;
use nom::{branch::alt, combinator::map, error::context, multi::many1, IResult};

use crate::assembler::SymbolTable;

use super::{directive_parsers::directive, instruction_parsers::instruction};

#[derive(Debug, PartialEq)]
pub struct Program {
    pub instructions: Vec<AssemblerInstruction>,
}

impl Program {
    pub fn to_bytes(&self, symbols: &SymbolTable) -> Vec<u8> {
        let mut program_bytes = vec![];
        for instruction in &self.instructions {
            program_bytes.append(&mut instruction.to_bytes(symbols));
        }
        program_bytes
    }
}

/// 解析输入字符串并返回一个程序结构。
///
/// 此函数是解析程序的入口点，它使用 Combinate 解析库中的 context, map, many1 和 alt 组合器
/// 来解析输入。该函数首先尝试匹配一系列指令或指令集，然后将这些指令封装到 Program 结构中。
///
/// 参数：
/// - input: &str - 待解析的输入字符串。
///
/// 返回：
/// - IResult<&str, Program> - 解析结果，包含解析得到的 Program 结构和剩余未解析的输入字符串。
pub fn program(input: &str) -> IResult<&str, Program> {
    // 使用 context 组合器为解析过程提供上下文信息，当解析失败时能够提供更丰富的错误信息。
    // 这里将上下文命名为"program"，以便在错误消息中标识出是在解析程序级别的结构。
    context(
        "program",
        // 使用 map 组合器将解析结果转换为 Program 结构。
        // many1 组合器用于解析一个或多个指令或指令集，alt 组合器用于在指令和指令集之间进行选择。
        map(many1(alt((instruction, directive))), |instructions| {
            // 将解析到的指令封装到 Program 结构中。
            Program { instructions }
        }),
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::assembler::SymbolTable;

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
        let symbols = SymbolTable::new();

        let result = program("load $0 #100\n");
        assert_eq!(result.is_ok(), true);
        let (_, program_res) = result.unwrap();
        let bytecode = program_res.to_bytes(&symbols);
        assert_eq!(bytecode.len(), 4);
        println!("load $0 #100  ==To_Bytes==> {:?}", bytecode);

        let result = program("load $0 #1000  \n   ");
        assert_eq!(result.is_ok(), true);
        let (_, program_res) = result.unwrap();
        let bytecode = program_res.to_bytes(&symbols);
        assert_eq!(bytecode.len(), 4);
        println!("load $0 #1000 ==To_Bytes==> {:?}", bytecode);
    }

    #[test]
    fn test_complete_program() {
        let test_program = "  .data\nhello: .asciiz 'Hello everyone!'\n.code\nhlt";
        let result = program(test_program);
        assert_eq!(result.is_ok(), true, "result:{:?}", result);
    }
}
