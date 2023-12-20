use std::{any, borrow::Cow};

use winnow::{
    ascii::{alphanumeric0, hex_digit1},
    combinator::{alt, dispatch, fail, not, opt, preceded, repeat, repeat_till0, success},
    stream::{AsChar, Stream},
    token::{any, take_while},
    PResult, Parser,
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Ident<'a> {
    data: Cow<'a, str>,
}

impl<'a> Ident<'a> {
    fn is_valid_start_char(c: char) -> bool {
        c.is_ascii_alphabetic() || c == '_' || !c.is_ascii()
    }

    fn is_valid_char(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '-' || c == '_' || !c.is_ascii()
    }

    fn eat_char(input: &mut &'a str) -> PResult<&'a str> {
        let empty = "";
        dispatch! {any;
            '\\' => parse_escaped_char,
            c if Self::is_valid_char(c) => {success(empty)},
            _ => fail::<_, &str, _>
        }
        .recognize()
        .parse_next(input)
    }

    pub fn parse(input: &mut &'a str) -> PResult<Self> {
        (
            alt((
                ("--"),
                ("-", take_while(1..=1, Self::is_valid_start_char)).recognize(),
                take_while(1..=1, Self::is_valid_start_char),
            )),
            repeat::<_, _, (), _, _>(0.., Self::eat_char),
        )
            .recognize()
            .map(|data: &'a str| Self { data: data.into() })
            .parse_next(input)
    }
}

fn hex_digit_token<'a>(input: &mut &'a str) -> PResult<&'a str> {
    (take_while(1..=6, AsChar::is_hex_digit), opt(" "))
        .recognize()
        .parse_next(input)
}

fn any_char<'a>(input: &mut &'a str) -> PResult<&'a str> {
    let src = *input;
    not((' ', '\r', '\t', '\n')).recognize().parse_next(input)
}

fn parse_escaped_char<'a>(input: &mut &'a str) -> PResult<&'a str> {
    let char_parser = alt((hex_digit_token, any_char));
    char_parser.recognize().parse_next(input)
}

#[cfg(test)]
mod test {
    use winnow::Parser;

    use super::Ident;

    #[test]
    pub fn test_valid_ident() {
        let input = "regular";
        let (rest, ident) = Ident::parse.parse_peek(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(ident.data, "regular");

        let input = "--fo-oA21f:bar";
        let (rest, ident) = Ident::parse.parse_peek(input).unwrap();
        assert_eq!(rest, ":bar");
        assert_eq!(ident.data, "--fo-oA21f");

        let input = "--23";
        let (rest, ident) = Ident::parse.parse_peek(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(ident.data, "--23");

        let input = "-fo-bar";
        let (rest, ident) = Ident::parse.parse_peek(input).unwrap();
        assert_eq!(rest, "");
        assert_eq!(ident.data, "-fo-bar");

        let input = "selector {asda}";
        let (rest, ident) = Ident::parse.parse_peek(input).unwrap();
        assert_eq!(rest, " {asda}");
        assert_eq!(ident.data, "selector");

        let input = "attr[xasd]";
        let (rest, ident) = Ident::parse.parse_peek(input).unwrap();
        assert_eq!(rest, "[xasd]");
        assert_eq!(ident.data, "attr");

        let input = "funct(foo)";
        let (rest, ident) = Ident::parse.parse_peek(input).unwrap();
        assert_eq!(rest, "(foo)");
        assert_eq!(ident.data, "funct");
    }

    #[test]
    pub fn test_valid_escape_ident() {
        let input = "foo\\nn ";

        let (rest, ident) = Ident::parse.parse_peek(input).unwrap();
        assert_eq!(ident.data, "foo\\nn");
        assert_eq!(rest, " ");

        let input = "foo\\AA ss ";

        let (rest, ident) = Ident::parse.parse_peek(input).unwrap();
        assert_eq!(ident.data, "foo\\AA ss");
        assert_eq!(rest, " ");
    }

    #[test]
    pub fn test_invalid_ident() {
        let input = "2-digits";
        let err = Ident::parse.parse_peek(input).unwrap_err();
        // panic!("{:?}", err)

        let input = "-2-digits";
        let err = Ident::parse.parse_peek(input).unwrap_err();
    }
}
