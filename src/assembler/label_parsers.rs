use nom::{
    bytes::complete::tag,
    character::complete::{alphanumeric1, char, multispace0},
    combinator::{map, map_res, opt},
    error::context,
    sequence::{preceded, tuple},
    IResult,
};

use super::Token;

/// Looks for a user-defined label, such as `label1:`
pub fn label_declaration(input: &str) -> IResult<&str, Token> {
    context(
        "label_declaration",
        preceded(
            multispace0,
            map_res(tuple((alphanumeric1, tag(":"))), |(lable, _)| {
                Ok::<Token, &str>(Token::LabelDeclaration {
                    name: String::from(lable),
                })
            }),
        ),
    )(input)
}

/// Looks for a user-defined label, such as `@label1`
pub fn label_usage(input: &str) -> IResult<&str, Token> {
    context(
        "label_usage",
        preceded(
            multispace0,
            map(
                tuple((char('@'), alphanumeric1, opt(multispace0))),
                |(_c, name, _)| Token::LabelUsage {
                    name: String::from(name),
                },
            ),
        ),
    )(input)
}

#[cfg(test)]
mod tests {
    use crate::assembler::Token;

    use super::{label_declaration, label_usage};

    #[test]
    fn test_parse_label_declaration() {
        let expect = Token::LabelDeclaration {
            name: "test".to_string(),
        };
        let result = label_declaration("test:\n");
        assert_eq!(result.is_ok(), true);
        let (_, token) = result.unwrap();
        assert_eq!(token, expect);
        let result = label_declaration("test\n");
        assert_eq!(result.is_ok(), false);
        let result = label_declaration("test :\n");
        assert_eq!(result.is_ok(), false);

        let expect = Token::LabelDeclaration {
            name: "112tes2t3".to_string(),
        };
        let result = label_declaration("   112tes2t3:   \n");
        assert_eq!(result.is_ok(), true);
        let (_, token) = result.unwrap();
        assert_eq!(token, expect);

        let result = label_declaration("   112tes2t3 :   \n");
        assert_eq!(result.is_ok(), false);
    }

    #[test]
    fn test_parse_label_usage() {
        let expect = Token::LabelUsage {
            name: "test".to_string(),
        };
        let result = label_usage("@test\n");
        assert_eq!(result.is_ok(), true);
        let (_, token) = result.unwrap();
        assert_eq!(token, expect);

        let result = label_usage("test");
        assert_eq!(result.is_ok(), false);

        let result = label_usage("@ test");
        assert_eq!(result.is_ok(), false);

        let result = label_usage(" @test  \n");
        assert_eq!(result.is_ok(), true);
        let (_, token) = result.unwrap();
        assert_eq!(token, expect);

        let expect = Token::LabelUsage {
            name: "1te12st".to_string(),
        };

        let result = label_usage("@1te12st\n");
        assert_eq!(result.is_ok(), true);
        let (_, token) = result.unwrap();
        assert_eq!(token, expect);

        let result = label_usage(" @1te12st  \n");
        assert_eq!(result.is_ok(), true);
        let (_, token) = result.unwrap();
        assert_eq!(token, expect);

        let result = label_usage(" @  1te12st \n");
        assert_eq!(result.is_ok(), false);
    }
}
