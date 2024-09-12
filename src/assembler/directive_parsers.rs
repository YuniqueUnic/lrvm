use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, line_ending, multispace0},
    combinator::{eof, map, map_res, opt},
    error::context,
    sequence::{preceded, terminated, tuple},
    IResult,
};

use super::{
    instruction_parsers::AssemblerInstruction, label_parsers::label_declaration,
    operand_parser::operand, Token,
};

pub fn directive_declaration(input: &str) -> IResult<&str, Token> {
    context(
        "directive_declaration",
        preceded(
            multispace0,
            map_res(preceded(tag("."), alpha1), |directive| {
                Ok::<Token, &str>(Token::Directive {
                    name: String::from(directive),
                })
            }),
        ),
    )(input)
}

fn directive_combined(input: &str) -> IResult<&str, AssemblerInstruction> {
    context(
        "directive_combined",
        preceded(
            multispace0,
            terminated(
                map(
                    tuple((
                        opt(label_declaration),
                        directive_declaration,
                        opt(operand),
                        opt(operand),
                        opt(operand),
                    )),
                    |(l, name, o1, o2, o3)| AssemblerInstruction {
                        opcode: None,
                        directive: Some(name),
                        label: l,
                        operand1: o1,
                        operand2: o2,
                        operand3: o3,
                    },
                ),
                alt((multispace0, line_ending, eof)),
            ),
        ),
    )(input)
}

pub fn directive(input: &str) -> IResult<&str, AssemblerInstruction> {
    context("directive", alt((directive_combined,)))(input)
}

#[cfg(test)]
mod tests {
    use crate::assembler::{instruction_parsers::AssemblerInstruction, Token};

    use super::{directive_combined, directive_declaration};

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

    #[test]
    fn test_string_directive() {
        let result = directive_combined("test: .asciiz 'Hello'");
        assert_eq!(result.is_ok(), true);
        let (_, directive) = result.unwrap();

        // Yes, this is the what the result should be
        let correct_instruction = AssemblerInstruction {
            opcode: None,
            label: Some(Token::LabelDeclaration {
                name: "test".to_string(),
            }),
            directive: Some(Token::Directive {
                name: "asciiz".to_string(),
            }),
            operand1: Some(Token::IrString {
                name: "Hello".to_string(),
            }),
            operand2: None,
            operand3: None,
        };

        assert_eq!(directive, correct_instruction);
    }
}
