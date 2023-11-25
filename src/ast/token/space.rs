// ┌───────────────────────────────────────────────────────────────────────────┐
// │                                                                           │
// │  ██████╗ ██████╗  ██████╗   Copyright (C) 2022, The Prospective Company   │
// │  ██╔══██╗██╔══██╗██╔═══██╗                                                │
// │  ██████╔╝██████╔╝██║   ██║  This file is part of the Procss library,      │
// │  ██╔═══╝ ██╔══██╗██║   ██║  distributed under the terms of the            │
// │  ██║     ██║  ██║╚██████╔╝  Apache License 2.0.  The full license can     │
// │  ╚═╝     ╚═╝  ╚═╝ ╚═════╝   be found in the LICENSE file.                 │
// │                                                                           │
// └───────────────────────────────────────────────────────────────────────────┘

use winnow::{
    ascii::multispace1,
    // branch::alt,
    // bytes::complete::{is_not, tag},
    // character::complete::{anychar, multispace1},
    combinator::{alt, not, preceded, repeat, repeat_till0},
    error::ParserError,
    // multi::{many0, many1, many_till},
    // sequence::preceded,
    token::{any, tag, take_till1},
    IResult,
    PResult,
    Parser,
};

/// An "extension" trait for [`str`] which is used frequently to determine
/// whether whitepsace can be removed during [`crate::render::RenderCss`]
pub trait NeedsWhitespaceStringExt {
    /// Does this string needs a leading whitespace character?
    fn needs_pre_ws(&self) -> bool;

    /// Does this string needs a trailing whitespace character?
    fn needs_post_ws(&self) -> bool;
}

impl NeedsWhitespaceStringExt for str {
    fn needs_pre_ws(&self) -> bool {
        self.chars()
            .next()
            .map(|x| x.is_ascii_alphanumeric() || x == '-' || x == '_' || x == '%' || x == '+')
            .unwrap_or_default()
    }

    fn needs_post_ws(&self) -> bool {
        self.chars()
            .last()
            .map(|x| x.is_ascii_alphanumeric() || x == '-' || x == '_' || x == '%' || x == '+')
            .unwrap_or_default()
    }
}

/// Render `s` trimming all intermediate whitespace to a single character along
/// the way.
pub fn trim_whitespace(s: &str, f: &mut std::fmt::Formatter<'_>) {
    let mut last_alpha = false;
    s.split_whitespace().for_each(|w| {
        if last_alpha && w.needs_pre_ws() {
            write!(f, " ").unwrap();
        }

        last_alpha = w.needs_post_ws();
        write!(f, "{}", w).unwrap();
    });
}

// pub fn trim_whitespace(s: &str, f: &mut std::fmt::Formatter<'_>) {
//     let mut flag = false;
//     s.split_whitespace().for_each(|w| {
//         if flag {
//             write!(f, " ").unwrap();
//         }

//         flag = flag || !w.is_empty();
//         write!(f, "{}", w).unwrap();
//     });
// }

fn parse_comment<'a, E>(input: &mut &'a str) -> PResult<(), E>
where
    E: ParserError<&'a str>,
{
    ignore(preceded(
        tag("//"),
        repeat::<_, _, Vec<_>, _, _>(0.., take_till1(('\r', '\n'))),
    ))
    .parse_next(input)
}

fn parse_multi_comment<'a, E>(input: &mut &'a str) -> PResult<(), E>
where
    E: ParserError<&'a str>,
{
    ignore(preceded(
        tag("/*"),
        repeat_till0::<_, _, Vec<_>, _, _, _, _>(any, tag("*/")),
    ))
    .parse_next(input)
}

fn ignore<'a, T, E, F>(mut f: F) -> impl Parser<&'a str, (), E>
where
    // F: FnMut(&mut &'a str) -> PResult<T, E>,
    F: Parser<&'a str, T, E>,
{
    move |input: &mut &'a str| {
        let _ = f.parse_next(input)?;
        Ok(())
    }
}

/// Parses 0 or more whitespace characters, including comments.
pub fn comment0<'a, E>(input: &mut &'a str) -> PResult<(), E>
where
    E: ParserError<&'a str>,
{
    repeat(
        0..,
        alt((ignore(multispace1), parse_comment, parse_multi_comment)),
    )
    .parse_next(input)?;
    Ok(())
}

/// Parses 1 or more whitespace characters, including comments.
pub fn comment1<'a, E>(input: &mut &'a str) -> PResult<(), E>
where
    E: ParserError<&'a str>,
{
    ignore(repeat::<_, _, Vec<_>, _, _>(
        1..,
        alt((ignore(multispace1), parse_comment, parse_multi_comment)),
    ))
    .parse_next(input)
}

/// Parses 0 or more whitespace characters, including comments and semicolons.
pub fn sep0<'a, E>(input: &mut &'a str) -> PResult<(), E>
where
    E: ParserError<&'a str>,
{
    ignore(repeat::<_, _, Vec<_>, _, _>(
        0..,
        alt((comment1, ignore(tag(";")))),
    ))
    .parse_next(input)
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::*;

    #[test]
    fn test_multiline_comment() {
        assert_matches!(
            comment0::<()>.parse_peek(
                "
    /* 
     * test
     */"
            ),
            Ok(("", ()))
        )
    }

    #[test]
    fn test_forward_slash() {
        assert_matches!(comment0::<()>.parse_peek("// test"), Ok(("", ())))
    }

    #[test]
    fn test_semicolons() {
        assert_matches!(comment0::<()>.parse_peek("/* test; test */"), Ok(("", ())))
    }
}
