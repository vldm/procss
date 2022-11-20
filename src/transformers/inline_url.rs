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

use std::borrow::Cow;
use std::path::Path;

use nom::branch::alt;
use nom::bytes::complete::{is_not, tag};
use nom::sequence::delimited;

use crate::ast::{Css, Rule};

#[cfg(feature = "iotest")]
mod fs {
    pub fn read_to_string(_input: &std::path::Path) -> Result<&'static str, String> {
        criterion::black_box(Ok(include_str!("../../benches/test.svg")))
    }
}

#[cfg(not(feature = "iotest"))]
mod fs {
    pub use std::fs::read_to_string;
}

fn parse_url(input: &str) -> nom::IResult<&str, &str> {
    let unquoted = delimited(tag("url("), is_not(")"), tag(")"));
    let quoted = delimited(tag("url(\""), is_not("\""), tag("\")"));
    alt((quoted, unquoted))(input)
}

fn into_data_uri<'a>(path: &Path) -> Cow<'a, str> {
    let contents = fs::read_to_string(path).expect("Error reading file");
    let encoded = base64::encode(contents);
    format!("url(data:image/svg+xml;base64,{})", encoded).into()
}

fn inline_url_impl<'a>(newpath: &str, flat: &mut Css<'a>) {
    flat.transform::<Rule<'a>>(|rule| {
        let path = parse_url(&rule.value)
            .ok()
            .and_then(|x| x.0.is_empty().then_some(Path::new(newpath).join(x.1)));

        if let Some(path) = &path {
            rule.value = into_data_uri(path);
        }
    })
}

/// Inline `url()` properties with the base64 encoded contents of their files.
pub fn inline_url<'a: 'b, 'b>(newpath: &'b str) -> impl Fn(&mut Css<'a>) + 'b {
    |flat| inline_url_impl(newpath, flat)
}
