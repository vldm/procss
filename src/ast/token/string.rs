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
    combinator::{alt, delimited, fold_repeat, not, preceded},
    error::{ErrorKind, ParserError},
    token::take_till1,
    IResult, PResult, Parser,
};

fn parse_escaped_char<'a>(input: &mut &'a str) -> PResult<&'a str> {
    preceded('\\', take_till1((' ', '\r', '\t', '\n', '\"')))
        .recognize()
        .parse_next(input)
}

fn parse_literal<'a>(input: &mut &'a str) -> PResult<&'a str> {
    let not_quote_slash = take_till1(('\"', '\\'));
    not_quote_slash
        .verify(|s: &str| !s.is_empty())
        .parse_next(input)
}

enum StringFragment {
    Literal(usize),
    EscapedChar(usize),
}

impl StringFragment {
    fn len(&self) -> usize {
        match self {
            StringFragment::Literal(s) => *s,
            StringFragment::EscapedChar(s) => *s,
        }
    }
}

fn parse_fragment<'a>(input: &mut &'a str) -> PResult<StringFragment> {
    alt((
        parse_literal.map(|x| StringFragment::Literal(x.len())),
        parse_escaped_char.map(|x| StringFragment::EscapedChar(x.len())),
    ))
    .parse_next(input)
}

pub fn parse_string_literal<'a, E: ParserError<&'a str>>() -> impl Parser<&'a str, &'a str, E> {
    move |input: &mut &'a str| {
        let build_string = fold_repeat(0.., parse_fragment, || 2, |len, frag| frag.len() + len);
        let offset = delimited('"', build_string, '"');
        let res = offset.map(|x| &input[..x]).parse_next(input);
        res.map_err(|_| {
            winnow::error::ErrMode::Backtrack(E::from_error_kind(&input, ErrorKind::Slice))
        })
    }
}
