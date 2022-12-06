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

#![feature(assert_matches)]

#[cfg(test)]
use std::assert_matches::assert_matches;

use procss::{parse, RenderCss};

#[test]
fn test_minify_percent_followed_by_minus() {
    assert_matches!(
        parse("div{width:calc(100% - 24px)}")
            .map(|x| x.as_css_string())
            .as_deref(),
        Ok("div{width:calc(100% - 24px);}")
    )
}

#[test]
fn test_minify_percent_followed_by_plus() {
    assert_matches!(
        parse("div{width:calc(100% + 28px)}")
            .map(|x| x.as_css_string())
            .as_deref(),
        Ok("div{width:calc(100% + 28px);}")
    )
}
