use crate::assembler::Token;

use nom::{
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
    combinator::map_res,
    sequence::preceded,
    IResult,
};

/// Parses a register token from the input string.
///
/// The register token starts with a '$' followed by at least one digit.
/// This function skips leading spaces and expects the token to be in this specific format.
///
/// # Arguments
/// * `input` - The input string to parse.
///
/// # Returns
/// * `IResult<&str, Token>` - The parsing result, either a Token with the parsed register number
///   or an error.
fn integer_operand(input: &str) -> IResult<&str, Token> {
    // Skip any leading spaces
    preceded(
        multispace0, // skip spaces first
        // Skip the '$' and read at least one digit
        map_res(
            preceded(tag("#"), digit1), // skip the $ first
            |reg_num: &str| {
                // Convert the string representation of the register number to an unsigned 8-bit integer
                Ok::<Token, &str>(Token::IntegerOperand {
                    value: reg_num.parse::<i32>().unwrap(),
                })
            },
        ),
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::assembler::Token;

    use super::integer_operand;

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
    }
}
