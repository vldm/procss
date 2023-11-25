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
use std::{collections::HashMap, path::Path};

use procss::{
    parse,
    transformers::{apply_import, apply_var},
    RenderCss,
};

#[test]
fn test_apply_import() {
    let mut trees = HashMap::default();
    trees.insert(
        Path::new("test"),
        parse("div.closed{color: green}").unwrap(),
    );

    let mut result = parse(
        "
            @import \"test\";
            div.open {
                color: red;
            }
        ",
    )
    .unwrap();
    apply_import(&trees)(&mut result);

    let result = result.flatten_tree().as_css_string();
    assert_matches!(&*result, "div.closed{color:green;}div.open{color:red;}")
}

#[test]
fn test_import_ref() {
    let mut trees = HashMap::default();
    trees.insert(
        Path::new("test"),
        parse("div.closed{color: ref}@green: #00FF00;").unwrap(),
    );
    assert_matches!(
        parse(
            "
            @import url(\"ref://test\");
            div.open {
                color: @green;
            }
        "
        )
        .map(|mut x| {
            apply_import(&trees)(&mut x);
            apply_var(&mut x);
            x.flatten_tree().as_css_string()
        })
        .as_deref(),
        Ok("div.open{color:#00FF00;}")
    )
}
