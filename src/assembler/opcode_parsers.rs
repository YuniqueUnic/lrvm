use crate::assembler::Token;
use crate::instruction::Opcode;
use nom::{character::complete::alpha1, combinator::map_res, error::context, IResult};

/// 解析 opcode 字符串
///
/// # 参数
/// * `input` - 待解析的 opcode 字符串切片
///
/// # 返回值
/// 返回一个`IResult<&str, Token>`，其中`Token`是解析后的 opcode 封装在`Token::Op`变体中
///
/// # 描述
/// 该函数使用`context`组合器设置错误上下文为"opcode"，并尝试使用`alt`组合器的备选方案解析输入字符串
/// 如果输入字符串以数字或字母开头，则使用`map_res`组合器映射结果为`Token::Op`变体，其中`code`字段是通过调用`opcode_from_str`
/// 函数从字符串转换得到的
///
/// # 例子
/// ```
/// let result = opcode("add");
/// assert_eq!(result, Ok((0, Token::Op { code: Some(OpCode::Add) })));
/// ```
pub fn opcode(input: &str) -> IResult<&str, Token> {
    context(
        "opcode",
        map_res(alpha1, |s: &str| {
            Ok::<Token, &str>(Token::Op {
                code: Opcode::from(s.to_lowercase().as_str()),
            })
        }),
    )(input)
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
        let result = opcode("load");
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(token, Token::Op { code: Opcode::LOAD });
        assert_eq!(rest, ""); // 剩余字符串

        // First tests that the opcode is detected and parsed correctly
        let result = opcode("LoAd");
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(token, Token::Op { code: Opcode::LOAD });
        assert_eq!(rest, ""); // 剩余字符串

        // Tests that an invalid opcode isn't recongized
        let result = opcode("aold");
        assert_eq!(result.is_ok(), true);
        let (_, token) = result.unwrap();
        assert_eq!(token, Token::Op { code: Opcode::IGL });
    }

    #[test]
    fn test_opcode_customize() {
        let result = opcode("load$1#2");
        assert_eq!(result.is_ok(), true);
        let (_rest, _token) = result.unwrap();
    }
}
