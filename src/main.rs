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

use std::path::Path;
use std::{env, fs};

use procss::{parse, RenderCss};

fn init() -> anyhow::Result<String> {
    let args: Vec<String> = env::args().collect();
    let contents = fs::read_to_string(Path::new(&args[1]));
    let css = parse(&contents?)?.flatten_tree().as_css_string();
    Ok(css)
}

fn main() {
    match init() {
        Ok(x) => println!("{}", x),
        Err(x) => eprintln!("{}", x),
    }
}
