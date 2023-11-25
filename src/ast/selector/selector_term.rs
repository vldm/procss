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
    combinator::{alt, delimited, opt, preceded, repeat},
    error::ParserError,
    token::tag,
    unpeek, IResult, Parser,
};

use super::attribute::SelectorAttr;
use crate::{ast::token::*, parser::*, render::*};

/// pseudo-selectors can be "pseudo-class" or "pseudo-element", and we are only
/// concerned about the distinction between them in regards to their syntax.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum PseudoMode {
    PseudoClass,
    PseudoElement,
}

/// A pseudo-selector component of a `Selector`, including optional argument
/// selector (parenthesis delimited).
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Pseudo<'a> {
    property: &'a str,
    value: Option<SelectorTerm<'a, Option<&'a str>>>,
    mode: PseudoMode,
}

impl<'a> ParseCss<'a> for Pseudo<'a> {
    fn parse<E: ParserError<&'a str>>(input: &'a str) -> IResult<&'a str, Self, E> {
        let (input, mode) = (tag(":"), opt(tag(":"))).parse_peek(input)?;
        let (input, property) = parse_symbol.parse_peek(input)?;
        let (input, value) =
            opt(delimited(tag("("), unpeek(SelectorTerm::parse), tag(")"))).parse_peek(input)?;
        let mode = mode
            .1
            .map(|_| PseudoMode::PseudoElement)
            .unwrap_or(PseudoMode::PseudoClass);

        Ok((input, Pseudo {
            property,
            value,
            mode,
        }))
    }
}

impl<'a> RenderCss for Pseudo<'a> {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.mode {
            PseudoMode::PseudoClass => write!(f, ":{}", self.property)?,
            PseudoMode::PseudoElement => write!(f, "::{}", self.property)?,
        };

        if let Some(x) = self.value.as_ref() {
            write!(f, "(")?;
            x.render(f)?;
            write!(f, ")")?;
        }

        Ok(())
    }
}

enum SelType<'a> {
    Class(&'a str),
    Id(&'a str),
    Pseudo(Pseudo<'a>),
    Attr(SelectorAttr<'a>),
}

/// A single compound CSS selector, parameterized over it's `tag` field such
/// that the uniqu wildcard and self selectors can re-use the same struct and
/// some tag-irrelevent functions can be shared between impls.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct SelectorTerm<'a, T> {
    pub id: Option<&'a str>,
    pub class: Vec<&'a str>,
    pub tag: T,
    pub attribute: Vec<SelectorAttr<'a>>,
    pub pseudo: Vec<Pseudo<'a>>,
}

impl<'a, T: Clone> SelectorTerm<'a, T> {
    /// Create a new `Selector`.
    fn new(tag: T, qualifiers: &[SelType<'a>]) -> SelectorTerm<'a, T> {
        let mut class = vec![];
        let mut id: Option<&str> = None;
        let mut attribute = vec![];
        let mut pseudo = vec![];
        for x in qualifiers {
            match x {
                SelType::Class(x) => class.push(*x),
                SelType::Id(x) => id = Some(x),
                SelType::Pseudo(x) => pseudo.push(x.clone()),
                SelType::Attr(x) => attribute.push(x.clone()),
            }
        }

        SelectorTerm {
            id,
            class,
            tag,
            attribute,
            pseudo,
        }
    }

    /// Join to another "self" selector.
    /// TODO Joining two selectors with populated `id` fields will discard the
    /// parent's `id`.
    pub fn join(&self, other: &SelectorTerm<'a, ()>) -> Self {
        let mut class = self.class.clone();
        let mut attribute = self.attribute.clone();
        let mut pseudo = self.pseudo.clone();
        let id = other.id.or(self.id);
        class.append(&mut other.class.clone());
        attribute.append(&mut other.attribute.clone());
        pseudo.append(&mut other.pseudo.clone());
        SelectorTerm {
            id,
            class,
            tag: self.tag.clone(),
            attribute,
            pseudo,
        }
    }
}

impl<'a, T: RenderCss> RenderCss for SelectorTerm<'a, T> {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.tag.render(f)?;
        if let Some(tag) = &self.id {
            write!(f, "#{}", tag)?;
        }

        for class in &self.class {
            write!(f, ".{}", class)?;
        }

        if !self.attribute.is_empty() {
            write!(f, "[")?;
            let mut first = true;
            for SelectorAttr { name, value } in &self.attribute {
                if !first {
                    write!(f, ",")?;
                }

                write!(f, "{}", name)?;
                if let Some(val) = value {
                    write!(f, "={}", val)?;
                }

                first = false;
            }

            write!(f, "]")?;
        }

        for class in &self.pseudo {
            class.render(f)?;
        }

        Ok(())
    }
}

// TODO multiple ids dont work correctly, we discard all but last

impl<'a> ParseCss<'a> for SelectorTerm<'a, Option<&'a str>> {
    fn parse<E>(input: &'a str) -> IResult<&'a str, Self, E>
    where
        E: ParserError<&'a str>,
    {
        let (rest, (tag, qualifiers)) = (
            opt(parse_symbol),
            repeat::<_, _, Vec<_>, _, _>(
                0..,
                alt((
                    preceded(tag("."), parse_symbol.map(SelType::Class)),
                    preceded(tag("#"), parse_symbol.map(SelType::Id)),
                    unpeek(Pseudo::parse).map(SelType::Pseudo),
                    unpeek(SelectorAttr::parse).map(SelType::Attr),
                )),
            ),
        )
            .parse_peek(input)?;

        if tag.is_none() && qualifiers.is_empty() {
            return winnow::IResult::Err(winnow::error::ErrMode::Backtrack(
                ParserError::from_error_kind(&rest, winnow::error::ErrorKind::Verify),
            ));
        }

        Ok((rest, SelectorTerm::new(tag, &qualifiers)))
    }
}

impl<'a> ParseCss<'a> for SelectorTerm<'a, ()> {
    fn parse<E>(input: &'a str) -> IResult<&'a str, Self, E>
    where
        E: ParserError<&'a str>,
    {
        let (rest, (_, qualifiers)) = (
            tag("&"),
            repeat::<_, _, Vec<_>, _, _>(
                0..,
                alt((
                    preceded(tag("."), parse_symbol.map(SelType::Class)),
                    preceded(tag("#"), parse_symbol.map(SelType::Id)),
                    unpeek(Pseudo::parse).map(SelType::Pseudo),
                    unpeek(SelectorAttr::parse).map(SelType::Attr),
                )),
            ),
        )
            .parse_peek(input)?;

        Ok((rest, SelectorTerm::new((), &qualifiers)))
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::*;

    #[test]
    fn test_tag() {
        assert_matches!(
            SelectorTerm::parse::<()>("--column-selector--background"),
            Ok(("", SelectorTerm {
                tag: Some("--column-selector--background"),
                ..
            }))
        )
    }

    #[test]
    fn test_class() {
        assert_matches!(
            SelectorTerm::<Option<&str>>::parse::<()>(".column-selector--background"),
            Ok(("", SelectorTerm {
                class,
                ..
            })) if class == vec!["column-selector--background"]
        )
    }

    #[test]
    fn test_classes() {
        assert_matches!(
            SelectorTerm::<Option<&str>>::parse::<()>(".column-selector.column-selector--background"),
            Ok(("", SelectorTerm {
                class,
                ..
            })) if class == vec!["column-selector", "column-selector--background"]
        )
    }

    #[test]
    fn test_attribute() {
        assert_matches!(
            SelectorTerm::<Option<&str>>::parse::<()>("[name=test]"),
            Ok(("", SelectorTerm {
                attribute,
                ..
            })) if attribute == vec![SelectorAttr{ name: "name", value: Some("test") }]
        )
    }

    #[test]
    fn test_id() {
        assert_matches!(
            SelectorTerm::<Option<&str>>::parse::<()>("#column-selector--background"),
            Ok(("", SelectorTerm {
                id: Some("column-selector--background"),
                ..
            }))
        )
    }

    #[test]
    fn test_id_class_tag() {
        assert_matches!(
            SelectorTerm::<Option<&str>>::parse::<()>("div#column-selector.column-selector.column-selector--background"),
            Ok(("", SelectorTerm {
                id: Some("column-selector"),
                class,
                tag: Some("div"),
                ..
            }))if class == vec!["column-selector", "column-selector--background"]
        )
    }

    #[test]
    fn test_pesudo() {
        assert_matches!(
            SelectorTerm::parse::<()>("div:hover"),
            Ok(("", SelectorTerm {
                tag: Some("div"),
                pseudo,
                ..
            })) if pseudo.len() == 1 && matches!(pseudo[0], Pseudo{property: "hover", value: None, mode: PseudoMode::PseudoClass })
        )
    }

    #[test]
    fn test_parameterized_pesudo() {
        assert_matches!(
            SelectorTerm::parse::<()>("div:not(.test)"),
            Ok(("", SelectorTerm {
                tag: Some("div"),
                pseudo,
                ..
            })) if pseudo.len() == 1 && matches!(pseudo[0], Pseudo{ property: "not", value: Some(_), mode: PseudoMode::PseudoClass })
        )
    }

    #[test]
    fn test_parameterized_pesudo_nth_child() {
        assert_matches!(
            SelectorTerm::parse::<()>("div:nth-child(2)"),
            Ok(("", SelectorTerm {
                tag: Some("div"),
                pseudo,
                ..
            })) if pseudo.len() == 1 && matches!(pseudo[0], Pseudo{ property: "nth-child", value: Some(_), mode: PseudoMode::PseudoClass })
        )
    }

    #[test]
    fn test_parameterized_pesudo_renders_correctly() {
        assert_matches!(
            SelectorTerm::<Option<&str>>::parse::<winnow::error::VerboseError<&str>>(
                "div:nth-child(2)"
            )
            .map(|x| x.as_css_string())
            .as_deref(),
            Ok("div:nth-child(2)")
        )
    }

    #[test]
    fn test_pesudo_element() {
        assert_matches!(
            SelectorTerm::parse::<()>("div::-webkit-scroll-thumb"),
            Ok(("", SelectorTerm {
                tag: Some("div"),
                pseudo,
                ..
            })) if pseudo.len() == 1 && matches!(pseudo[0], Pseudo{property: "-webkit-scroll-thumb", value: None, mode: PseudoMode::PseudoElement })
        )
    }

    #[test]
    fn test_pesudo_element_renders_correctly() {
        assert_matches!(
            SelectorTerm::<Option<&str>>::parse::<winnow::error::VerboseError<&str>>(
                "div::-webkit-scroll-thumb"
            )
            .map(|x| x.as_css_string())
            .as_deref(),
            Ok("div::-webkit-scroll-thumb")
        )
    }
}
