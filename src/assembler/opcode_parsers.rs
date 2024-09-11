use crate::assembler::Token;
use crate::instruction::Opcode;
use nom::{
    branch::alt,
    character::complete::{alpha1, digit1},
    combinator::map_res,
    IResult,
};

/// 将 &str 转换为 Opcode 枚举
fn opcode_from_str(input: &str) -> Opcode {
    match input {
        "load" => Opcode::LOAD,
        // 添加其他的 opcode 映射
        _ => Opcode::IGL, // 默认无效指令
    }
}

/// 解析 opcode 字符串
pub fn opcode_load(input: &str) -> IResult<&str, Token> {
    map_res(alt((digit1, alpha1)), |s: &str| {
        Ok::<Token, &str>(Token::Op {
            code: opcode_from_str(s),
        })
    })(input)
}

// 测试用例
#[cfg(test)]
mod tests {

    use super::*;
    use crate::assembler::Token;
    use crate::instruction::Opcode;

    #[test]
    fn test_opcode() {
        // First tests that the opcode is detected and parsed correctly
        let result = opcode_load("load");
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(token, Token::Op { code: Opcode::LOAD });
        assert_eq!(rest, ""); // 剩余字符串

        // Tests that an invalid opcode isn't recongized
        let result = opcode_load("aold");
        assert_eq!(result.is_ok(), true);
        let (_, token) = result.unwrap();
        assert_eq!(token, Token::Op { code: Opcode::IGL });
    }

    #[test]
    fn test_opcode_customize() {
        let result = opcode_load("load$1#2");
        println!("result: {:?}", result);
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        println!("rest: {:?}, token: {:?}", rest, token);
    }
}
