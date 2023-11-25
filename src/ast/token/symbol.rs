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
    ascii::alphanumeric1,
    combinator::{alt, repeat},
    error::ParserError,
    token::tag,
    PResult, Parser,
};

pub fn parse_symbol<'a, E>(input: &mut &'a str) -> PResult<&'a str, E>
where
    E: ParserError<&'a str>,
{
    let mut parser = repeat::<_, _, Vec<_>, _, _>(
        1..,
        alt((alphanumeric1, tag("-"), tag("_"), tag("*"), tag("%"))),
    )
    .recognize();
    parser.parse_next(input)
}
