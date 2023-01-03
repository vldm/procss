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

use std::process::Command;

const JS_TEST: &str = "
async function run() {
    const { BuildCss } = await import(\"@prospective.co/procss/target/cjs/procss.js\");
    const builder = new BuildCss(\"./virtual\");
    builder.add(\"test.scss\", \"div { span { color: red }}\");
    console.log(builder.compile().get(\"test.css\"));
}

run();
";

#[test]
fn test_apply_import() {
    let output = Command::new("node")
        .args(["-e", JS_TEST])
        .output()
        .expect("No output");

    assert_eq!(
        std::str::from_utf8(&output.stdout).unwrap(),
        "div span{color:red;}\n"
    );
}
