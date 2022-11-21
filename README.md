# <b>PRO</b><i>CSS</i>

[![CI](https://github.com/ProspectiveCo/procss/actions/workflows/build_dev.yaml/badge.svg)](https://github.com/ProspectiveCo/procss/actions/workflows/build_dev.yaml)

A simple CSS parsing and transformation framework.

[Docs](https://crates.io/crates/procss)

## Developer Setup

Build

```bash
cargo build --release
```

Test

```bash
cargo test --features iotest
```

Lint

```bash
cargo clippy
cargo fmt
```

Bench

```bash
cargo bench --features iotest -- --save-baseline my-baseline
```

Generate docs (output at `./target/doc/procss/index.html`)

```bash
cargo doc
```
