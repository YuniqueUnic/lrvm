use nom::{
    branch::alt,
    character::complete::{alpha1, char, multispace0},
    combinator::{cut, map, map_res, opt},
    error::context,
    sequence::{preceded, tuple},
    IResult,
};

use super::{instruction_parsers::AssemblerInstruction, operand_parser::operand, Token};

pub fn directive_declaration(input: &str) -> IResult<&str, Token> {
    context(
        "directive_declaration",
        preceded(
            multispace0,
            map_res(cut(preceded(char('.'), alpha1)), |directive| {
                Ok::<Token, &str>(Token::Directive {
                    name: String::from(directive),
                })
            }),
        ),
    )(input)
}

pub fn directive_combined(input: &str) -> IResult<&str, AssemblerInstruction> {
    context(
        "directive_combined",
        preceded(
            multispace0,
            map(
                tuple((
                    char('.'),
                    directive_declaration,
                    opt(operand),
                    opt(operand),
                    opt(operand),
                )),
                |(_, name, o1, o2, o3)| AssemblerInstruction {
                    opcode: None,
                    label: None,
                    directive: Some(name),
                    operand1: o1,
                    operand2: o2,
                    operand3: o3,
                },
            ),
        ),
    )(input)
}

pub fn directive(input: &str) -> IResult<&str, AssemblerInstruction> {
    context("directive", alt((directive_combined,)))(input)
}

#[cfg(test)]
mod tests {
    use crate::assembler::Token;

    use super::directive_declaration;

    #[test]
    fn test_directive() {
        let expect = Token::Directive {
            name: String::from("data"),
        };

        let result = directive_declaration(".data\n");
        assert!(result.is_ok(), "directive: {:?}", result);
        let (_, token) = result.unwrap();
        assert_eq!(expect, token);

        let result = directive_declaration("   .data   \n  ");
        assert!(result.is_ok(), "directive: {:?}", result);
        let (_, token) = result.unwrap();
        assert_eq!(expect, token);
    }
}
