use crate::assembler::Token;

use nom::{
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
    combinator::map_res,
    error::context,
    sequence::preceded,
    IResult,
};

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
pub fn integer_operand(input: &str) -> IResult<&str, Token> {
    context(
        "integer_operand",
        // Skip any leading spaces
        preceded(
            multispace0, // skip spaces first
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

        let result = integer_operand("# 10");
        assert_eq!(result.is_ok(), false);
    }
}
