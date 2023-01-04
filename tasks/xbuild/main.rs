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

use std::env::args;
use std::process::Command;

use once_cell::sync::Lazy;
use procss_utils::*;
use wasm_opt::OptimizationOptions;

static IS_RELEASE: Lazy<bool> = Lazy::new(|| args().any(|x| x == "--release"));

fn build_cmd<'a, I: IntoIterator<Item = &'a str>>(args: I) -> Command {
    let mut cmd = Command::new("cargo");
    if *IS_RELEASE {
        cmd.args(["build", "--release"].into_iter().chain(args));
    } else {
        cmd.args(["build"].into_iter().chain(args));
    }

    cmd
}

fn wasm_bindgen_cmd<'a, I: IntoIterator<Item = &'a str>>(args: I) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_FILE_WASM_BINDGEN_CLI_wasm-bindgen"));
    let wasm = if *IS_RELEASE {
        "target/wasm32-unknown-unknown/release/procss.wasm"
    } else {
        "target/wasm32-unknown-unknown/debug/procss.wasm"
    };

    cmd.args([wasm].into_iter().chain(args));
    cmd
}

fn main() {
    let target = get_default_target();
    build_cmd(["--target", target]).execute();
    build_cmd(["--target", "wasm32-unknown-unknown"]).execute();
    wasm_bindgen_cmd(["--out-dir", "target/esm", "--target", "web"]).execute();
    wasm_bindgen_cmd(["--out-dir", "target/cjs", "--target", "nodejs"]).execute();
    if *IS_RELEASE {
        OptimizationOptions::new_optimize_for_size_aggressively()
            .one_caller_inline_max_size(15)
            .run("target/esm/procss_bg.wasm", "target/esm/procss_bg.wasm")
            .unwrap();

        OptimizationOptions::new_optimize_for_size_aggressively()
            .one_caller_inline_max_size(15)
            .run("target/cjs/procss_bg.wasm", "target/cjs/procss_bg.wasm")
            .unwrap();
    }
}
