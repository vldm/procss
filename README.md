# <b>PRO</b><i>CSS</i>

A simple CSS parsing and transformation framework.

## Developer Setup

Build

```bash
cargo build --release
```

Test

```bash
cargo clippy
cargo test --doc --features iotest
cargo nextest run
```

Bench

```bash
cargo bench --features iotest -- --save-baseline my-baseline
```

Generate docs (output at `./target/doc/procss/index.html`)

```bash
cargo doc
```
