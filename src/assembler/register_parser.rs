use crate::assembler::Token;

use nom::{
    bytes::complete::tag,
    character::complete::{digit1, multispace0},
    combinator::map_res,
    error::context,
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
pub fn register(input: &str) -> IResult<&str, Token> {
    context(
        "register",
        // Skip any leading spaces
        preceded(
            multispace0, // skip spaces first
            // Skip the '$' and read at least one digit
            map_res(
                preceded(tag("$"), digit1), // skip the $ first
                |reg_num: &str| {
                    // Convert the string representation of the register number to an unsigned 8-bit integer
                    Ok::<Token, &str>(Token::Register {
                        reg_num: reg_num.parse::<u8>().unwrap(),
                    })
                },
            ),
        ),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::register;

    #[test]
    fn test_parse_register() {
        let result = register("$0");
        assert_eq!(result.is_ok(), true);
        let result = register("0");
        assert_eq!(result.is_ok(), false);
        let result = register("$a");
        assert_eq!(result.is_ok(), false);
        let result = register("$ 100");
        assert_eq!(result.is_ok(), false);
    }
}
