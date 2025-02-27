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

use winnow::{
    combinator::{alt, not, repeat},
    error::ParserError,
    token::{tag, take_till1},
    IResult, Parser,
};

use crate::{
    ast::token::{comment0, parse_string_literal, parse_symbol, trim_whitespace},
    render::RenderCss,
    transform::TransformCss,
};

/// A CSS rule, of the form `xxx: yyy` (delimited by `;` in a ruleset).
#[derive(Clone, Debug)]
pub struct Rule<'a> {
    pub property: Cow<'a, str>,
    pub value: Cow<'a, str>,
}

impl<'a> TransformCss<Rule<'a>> for Rule<'a> {
    fn transform_each<F: FnMut(&mut Rule<'a>)>(&mut self, f: &mut F) {
        f(self)
    }
}

impl<'a> RenderCss for Rule<'a> {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Rule { property, value } = self;
        write!(f, "{}:", property)?;
        trim_whitespace(value, f);
        write!(f, ";")
    }
}

// TODO property is not the same parser as tag.
// TODO this Cow is not borrowed ...
impl<'a> crate::parser::ParseCss<'a> for Rule<'a> {
    fn parse<E: ParserError<&'a str>>(input: &'a str) -> IResult<&'a str, Self, E> {
        let (input, property) = parse_symbol.parse_peek(input)?;
        let (input, _) = Parser::parse_peek(&mut (comment0, tag(":"), comment0), input)?;
        let (input, value) = repeat::<_, _, Vec<_>, _, _>(
            0..,
            alt((take_till1(('\"', ';', '}')), parse_string_literal())),
        )
        .recognize()
        .parse_peek(input)?;
        Ok((input, Rule {
            property: property.into(),
            value: value.into(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::*;
    use crate::parser::ParseCss;

    #[test]
    fn test_rule_value_string() {
        assert_matches!(
            Rule::parse::<()>("--column-selector--background: url(\"test\")"),
            Ok(("", Rule {
                property,
                value,
            })) if value == "url(\"test\")" && property == "--column-selector--background"
        )
    }

    #[test]
    fn test_rule_escaped_string() {
        assert_matches!(
            Rule::parse::<()>("test: \"\\1234\""),
            Ok(("", Rule {
                property,
                value,
            })) if value == "\"\\1234\"" && property == "test"
        )
    }

    #[test]
    fn test_rule_escaped_string_2() {
        assert_matches!(
            Rule::parse::<()>("test: \": test ; alpha\""),
            Ok(("", Rule {
                property,
                value,
            })) if value == "\": test ; alpha\"" && property ==  "test"
        )
    }

    #[ignore]
    #[test]
    fn test_rule_escaped_string_3() {
        assert_matches!(
            Rule::parse::<()>("test: ': test ; alpha'"),
            Ok(("", Rule {
                property,
                value,
            })) if value == "\"\\1234\"" && property == "test"
        )
    }
}
