use core::num;

use crate::assembler::Token;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{char, digit1, line_ending, multispace0},
    combinator::{eof, map_res, opt},
    error::context,
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

use super::{label_parsers::label_usage, register_parser::register};

/// Parses an integer operand from a string.
///
/// This function expects the input string to contain an integer operand prefixed by a '#'.
/// It skips leading spaces, then reads the '#' followed by at least one digit.
///
/// # Arguments
/// * `input` - A string potentially containing an integer operand.
///
/// # Returns
/// * `IResult<&str, Token>` - A result containing either a `Token` representing the integer operand
///   or an error, along with any remaining unparsed input string.
fn integer_operand(input: &str) -> IResult<&str, Token> {
    context(
        "integer_operand",
        // Skip any leading spaces
        preceded(
            multispace0, // skip spaces first
            terminated(
                // Skip the '#' and read at least one digit
                map_res(
                    preceded(tag("#"), digit1), // skip the # first
                    |reg_num: &str| {
                        // Convert the string representation of the number to an i32 and create a Token::IntegerOperand
                        Ok::<Token, &str>(Token::IntegerOperand {
                            value: reg_num.parse::<i32>().unwrap(),
                        })
                    },
                ),
                alt((multispace0, line_ending, eof)),
            ),
        ),
    )(input)
}

fn float_operand(input: &str) -> IResult<&str, Token> {
    context(
        "float_operand",
        // Skip any leading spaces
        preceded(
            multispace0, // skip spaces first
            terminated(
                // Skip the '#' and read at least one digit
                map_res(
                    preceded(tag("#"), tuple((opt(char('-')), digit1, char('.'), digit1))), // skip the # first
                    |(sign, left, dot, right)| {
                        let mut num_str = String::from(left);
                        num_str.push(dot);
                        num_str.push_str(right);

                        let converted = match num_str.parse::<f64>() {
                            Ok(n) => n,
                            Err(e) => {
                                eprintln!("Failed to parse float: {}", e);
                                return Err(input);
                            },
                        };
                        let value = if sign.is_some() {
                            -converted
                        } else {
                            converted
                        };

                        // Convert the string representation of the number to an i32 and create a Token::IntegerOperand
                        Ok::<Token, &str>(Token::Factor {
                            value: Box::new(Token::Float { value }),
                        })
                    },
                ),
                alt((multispace0, line_ending, eof)),
            ),
        ),
    )(input)
}

fn ir_string_single_quota(input: &str) -> IResult<&str, Token> {
    context(
        "ir_string_single_quota",
        preceded(
            multispace0,
            terminated(
                delimited(tag("'"), take_while(|c: char| c != '\''), tag("'")),
                alt((multispace0, line_ending, eof)),
            ),
        ),
    )(input)
    .map(|(rest, content): (_, &str)| {
        (
            rest,
            Token::IrString {
                name: content.to_string(),
            },
        )
    })
}

fn ir_string_double_quota(input: &str) -> IResult<&str, Token> {
    context(
        "ir_string_double_quota",
        preceded(
            multispace0,
            terminated(
                delimited(tag("\""), take_while(|c: char| c != '\"'), tag("\"")),
                alt((multispace0, line_ending, eof)),
            ),
        ),
    )(input)
    .map(|(rest, content): (_, &str)| {
        (
            rest,
            Token::IrString {
                name: content.to_string(),
            },
        )
    })
}

pub fn ir_string(input: &str) -> IResult<&str, Token> {
    context(
        "ir_string",
        alt((ir_string_single_quota, ir_string_double_quota)),
    )(input)
}

pub fn operand(input: &str) -> IResult<&str, Token> {
    context(
        "operand",
        alt((
            integer_operand,
            float_operand,
            label_usage,
            // label_declaration,
            register,
            ir_string,
        )),
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::assembler::Token;

    use super::{
        float_operand, integer_operand, ir_string, ir_string_double_quota, ir_string_single_quota,
    };

    #[test]
    fn test_parse_register() {
        let result = integer_operand("#10");
        assert_eq!(result.is_ok(), true);
        let (rest, value) = result.unwrap();
        assert_eq!(rest, "");
        assert_eq!(value, Token::IntegerOperand { value: 10 });
        let result = integer_operand("10");
        assert_eq!(result.is_ok(), false);
        let result = integer_operand("#a");
        assert_eq!(result.is_ok(), false);

        let result = integer_operand("# 10");
        assert_eq!(result.is_ok(), false);
    }

    #[test]
    fn test_ir_string_single_quota() {
        let input = "'Hello World'";
        let result = ir_string_single_quota(input);
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            token,
            Token::IrString {
                name: "Hello World".to_string()
            },
            "Token:{:?}",
            token
        );

        let input = "'With Spaces and !@#$%^&*()_+'";
        let result = ir_string_single_quota(input);
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            token,
            Token::IrString {
                name: "With Spaces and !@#$%^&*()_+".to_string()
            },
            "Token:{:?}",
            token
        );
    }

    #[test]
    fn test_ir_string_double_quota() {
        let input = "\"SingleWord\"";
        let result = ir_string_double_quota(input);
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            token,
            Token::IrString {
                name: "SingleWord".to_string()
            },
            "Token:{:?}",
            token
        );

        let input = "\"With Spaces and \"'quoted'\" strings\"";
        let result = ir_string_double_quota(input);
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(rest, "'quoted'\" strings\"");
        assert_eq!(
            token,
            Token::IrString {
                name: "With Spaces and ".to_string()
            },
            "Token:{:?}",
            token
        );
    }

    #[test]
    fn test_ir_string() {
        let input = "  'Hello World' \n";
        let result = ir_string(input);
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            token,
            Token::IrString {
                name: "Hello World".to_string()
            },
            "Token:{:?}",
            token
        );

        let input = "  \"SingleWord\"  ";
        let result = ir_string(input);
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            token,
            Token::IrString {
                name: "SingleWord".to_string()
            },
            "Token:{:?}",
            token
        );

        let input = " 'With Spaces and !@#$%^&*()_+' \n";
        let result = ir_string(input);
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            token,
            Token::IrString {
                name: "With Spaces and !@#$%^&*()_+".to_string()
            },
            "Token:{:?}",
            token
        );

        let input = "\"With Spaces and \"'quoted'\" strings\"\n";
        let result = ir_string(input);
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(rest, "'quoted'\" strings\"\n");
        assert_eq!(
            token,
            Token::IrString {
                name: "With Spaces and ".to_string()
            },
            "Token:{:?}",
            token
        );

        let input = "  \"With Spaces' and !@#$%^&*()_+\" \n";
        let result = ir_string(input);
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            token,
            Token::IrString {
                name: "With Spaces' and !@#$%^&*()_+".to_string()
            },
            "Token:{:?}",
            token
        );

        let input = "  'With Spaces\" and !@#$%^&*()_+' \n";
        let result = ir_string(input);
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(rest, "");
        assert_eq!(
            token,
            Token::IrString {
                name: "With Spaces\" and !@#$%^&*()_+".to_string()
            },
            "Token:{:?}",
            token
        );
    }

    #[test]
    fn test_parse_float_operand() {
        let _ = vec!["#100.3", "#-100.3", "#1.0", "#0.0"].iter().map(|i| {
            assert_eq!(float_operand(i).is_ok(), true);
        });
    }
}
