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

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use procss::{parse, parse_unchecked, transformers, RenderCss};

static CSS: &str = include_str!("./test.less");

fn test_overall(c: &mut Criterion) {
    c.bench_function("main()", |b| {
        b.iter(|| {
            let parsed = parse(black_box(CSS));
            let _css = parsed.unwrap().flatten_tree().as_css_string();
        })
    });
}

fn test_overall_fast(c: &mut Criterion) {
    c.bench_function("main_fast()", |b| {
        b.iter(|| {
            let parsed = parse_unchecked(black_box(CSS));
            let _css = parsed.unwrap().flatten_tree().as_css_string();
        })
    });
}

fn test_parse(c: &mut Criterion) {
    c.bench_function("parse()", |b| {
        b.iter(|| {
            let _x = parse(black_box(CSS));
        })
    });
}

fn test_parse_fast(c: &mut Criterion) {
    c.bench_function("parse_unchecked()", |b| {
        b.iter(|| {
            let _x = parse_unchecked(black_box(CSS));
        })
    });
}

fn test_flatten(c: &mut Criterion) {
    let parsed = parse(black_box(CSS));
    let css = parsed.unwrap();
    c.bench_function("flatten()", |b| {
        b.iter(|| {
            let _x = black_box(&css).flatten_tree();
        })
    });
}

fn test_render(c: &mut Criterion) {
    let parsed = parse(black_box(CSS));
    let css = parsed.unwrap();
    let css = css.flatten_tree();
    c.bench_function("as_css_string()", |b| {
        b.iter(|| {
            let _x = css.as_css_string();
        })
    });
}

fn test_inline(c: &mut Criterion) {
    let parsed = parse(black_box(CSS)).unwrap();
    let transform = transformers::inline_url("test");
    c.bench_function("inline_url()", |b| {
        let mut css = parsed.flatten_tree();
        b.iter(|| {
            transform(&mut css);
        })
    });
}

criterion_group!(overall, test_overall, test_overall_fast);
criterion_group!(parser, test_parse, test_parse_fast);
criterion_group!(other, test_flatten, test_render, test_inline);
criterion_main!(overall, parser, other);

// `bench` feature flag stubs out disk-accessing and other performance
// neutering function
#[cfg(all(not(feature = "bench"), not(debug_assertions)))]
compile_error!(
    "Feature 'bench' must be enabled - rerun like this:\n\n    cargo bench --features bench\n\n"
);
