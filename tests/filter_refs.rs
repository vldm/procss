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

use procss::transformers::filter_refs;
use procss::{ast, parse};

#[test]
fn test_filter_refs() {
    assert_matches!(
        parse(
            "
            div {
                color: green;
            }

            @red: #ff0000;

            @font-face {
                name: \"test\";
            }

            @media (min-width: 100px) {
                @media (max-width: 100px) {
                    div {
                        color: green;
                    }
                }
            }
        "
        )
        .map(|mut x| {
            filter_refs(&mut x);
            x
        }),
        Ok(ast::Tree(x)) if x.len() == 3
    )
}
