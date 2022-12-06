use std::process::{exit, Command};

trait SimpleCommand {
    fn execute(&mut self);
}

impl SimpleCommand for Command {
    fn execute(&mut self) {
        let code = self.status().expect("Failed to execute").code();
        match code {
            Some(x) if x == 0 => (),
            Some(x) => exit(x),
            None => exit(1),
        }
    }
}

fn main() {
    std::fs::remove_dir_all("target/prof").unwrap_or_default();
    let rust_flags = [
        "-Cinstrument-coverage",
        "-Ccodegen-units=1",
        "-Copt-level=0",
        "-Clink-dead-code",
        "-Coverflow-checks=off",
        "-Zpanic_abort_tests",
        "-Cpanic=abort",
    ];

    Command::new("cargo")
        .args(["test", "--features", "iotest"])
        .env("RUSTFLAGS", rust_flags.join(" "))
        .env("RUSTDOCFLAGS", "-Cpanic=abort")
        .env("CARGO_INCREMENTAL", "0")
        .env("LLVM_PROFILE_FILE", "target/prof/cargo-test-%p-%m.profraw")
        .execute();

    let grcov_args = [
        "./target/prof",
        "--binary-path",
        "./target/debug/deps/",
        "-s",
        ".",
        "--branch",
        "--ignore-not-existing",
        "--ignore",
        "../*",
        "--ignore",
        "/*",
    ];

    std::fs::remove_dir_all("target/coverage/html").unwrap_or_default();
    std::fs::create_dir_all("target/coverage/html").unwrap_or_default();
    let html_args = ["-t", "html", "-o", "target/coverage/html"];
    Command::new("grcov")
        .args(grcov_args.into_iter().chain(html_args))
        .execute();

    std::fs::remove_dir_all("target/coverage/lcov").unwrap_or_default();
    std::fs::create_dir_all("target/coverage/lcov").unwrap_or_default();
    let lcov_args = ["-t", "lcov", "-o", "target/coverage/lcov/lcov.info"];
    let code = Command::new("grcov")
        .args(grcov_args.into_iter().chain(lcov_args))
        .status()
        .expect("failed to execute process")
        .code();

    exit(code.unwrap());
}
