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
fn test_css() {
    assert_matches!(
        parse("div{color:red}")
            .map(|x| x.as_css_string())
            .as_deref(),
        Ok("div{color:red;}")
    )
}

#[test]
fn test_multiple_rules() {
    assert_matches!(
        parse("div{color:red;font:sans}")
            .map(|x| x.as_css_string())
            .as_deref(),
        Ok("div{color:red;font:sans;}")
    )
}

#[test]
fn test_multiple_blocks() {
    assert_matches!(
        parse("div{color:red}a{color:green}")
            .map(|x| x.as_css_string())
            .as_deref(),
        Ok("div{color:red;}a{color:green;}")
    )
}

#[test]
fn test_whitspace() {
    assert_matches!(
        parse(" div { color : red } a { color : green } ")
            .map(|x| x.as_css_string())
            .as_deref(),
        Ok("div{color:red;}a{color:green;}")
    )
}

#[test]
fn test_whitspace2() {
    assert_matches!(
        parse(
            " 
        div { 
            color: red;
        } 
        
        a {
            color: green
        }
        "
        )
        .map(|x| x.as_css_string())
        .as_deref(),
        Ok("div{color:red;}a{color:green;}")
    )
}

#[test]
fn test_multiple_pseudo() {
    assert_matches!(
        parse("div:hover:first-child{color:red}")
            .map(|x| x.flatten_tree().as_css_string())
            .as_deref(),
        Ok("div:hover:first-child{color:red;}")
    )
}

#[test]
fn test_selector_attribute_pseudo_correct_order() {
    let complex = "
        div[data-value]:before {
            color: red;
        }
    ";

    assert_matches!(
        parse(complex)
            .map(|x| x.flatten_tree().as_css_string())
            .as_deref(),
        Ok("div[data-value]:before{color:red;}")
    )
}
