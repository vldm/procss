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
    combinator::{alt, delimited},
    error::ParserError,
    token::tag,
    IResult, Parser,
};

use crate::{ast::token::*, parser::*, render::*};

/// A selector combinator, used to combine a list of selectors.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Combinator {
    Null,
    Sibling,
    AdjSibling,
    Desc,
}

impl RenderCss for Combinator {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Combinator::Null => write!(f, " "),
            Combinator::Sibling => write!(f, "~"),
            Combinator::AdjSibling => write!(f, "+"),
            Combinator::Desc => write!(f, ">"),
        }
    }
}

impl<'a> ParseCss<'a> for Combinator {
    fn parse<E>(input: &'a str) -> IResult<&'a str, Self, E>
    where
        E: ParserError<&'a str>,
    {
        delimited(
            comment0,
            alt((
                tag("+").map(|_| Combinator::AdjSibling),
                tag(">").map(|_| Combinator::Desc),
                tag("~").map(|_| Combinator::Sibling),
                comment0.map(|_| Combinator::Null),
            )),
            comment0,
        )
        .parse_peek(input)
    }
}
