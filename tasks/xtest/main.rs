use std::process::{exit, Command};

fn main() {
    std::fs::remove_dir_all("target/prof").unwrap_or_default();
    let code = Command::new("cargo")
        .args(["test", "--features", "iotest"])
        .env(
            "RUSTFLAGS",
            "-Cinstrument-coverage -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code \
             -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort",
        )
        .env("RUSTDOCFLAGS", "-Cpanic=abort")
        .env("CARGO_INCREMENTAL", "0")
        .env("LLVM_PROFILE_FILE", "target/prof/cargo-test-%p-%m.profraw")
        .status()
        .expect("failed to execute process")
        .code();

    if matches!(code, Some(x) if x > 0) {
        exit(code.unwrap());
    }

    std::fs::remove_dir_all("target/coverage/html").unwrap_or_default();
    std::fs::create_dir_all("target/coverage/html").unwrap_or_default();
    let code = Command::new("grcov")
        .args([
            "./target/prof",
            "--binary-path",
            "./target/debug/deps/",
            "-s",
            ".",
            "-t",
            "html",
            "--branch",
            "--ignore-not-existing",
            "--ignore",
            "../*",
            "--ignore",
            "/*",
            "-o",
            "target/coverage/html",
        ])
        .status()
        .expect("failed to execute process")
        .code();

    std::fs::remove_dir_all("target/coverage/lcov").unwrap_or_default();
    std::fs::create_dir_all("target/coverage/lcov").unwrap_or_default();
    let code = Command::new("grcov")
        .args([
            "./target/prof",
            "--binary-path",
            "./target/debug/deps/",
            "-s",
            ".",
            "-t",
            "lcov",
            "--branch",
            "--ignore-not-existing",
            "--ignore",
            "../*",
            "--ignore",
            "/*",
            "-o",
            "target/coverage/lcov/lcov.info",
        ])
        .status()
        .expect("failed to execute process")
        .code();

    exit(code.unwrap());
}
