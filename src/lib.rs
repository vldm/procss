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

//! A simple CSS parsing and transformation framework. Procss can be used to
//! quickly bundle a collection of CSS+ files, or write your own custom
//! transforms.
//!
//! # Usage
//!
//! Procss's parser understands a nested superset of CSS (which we refer to as
//! CSS+), similar to the [CSS nesting proposal](https://www.w3.org/TR/css-nesting-1/),
//! or languages like [Sass](https://sass-lang.com).  Start with source CSS+
//! as a [`str`], use [crate::parse] or [crate::parse_unchecked] to generate
//! an [`ast::Tree`].
//!
//! ```
//! use procss::{ast, parse};
//!
//! let ast = procss::parse("div{.open{color:red;}}").unwrap();
//! ```
//!
//! The resulting [`ast::Tree`] can be converted to a de-nested [`ast::Css`]
//! with the [`ast::Tree::flatten_tree`] method, which itself can then be
//! rendered as a plain browser-readable CSS string via the
//! [`RenderCss::as_css_string`] trait method.
//!
//! ```
//! # use procss::{parse, ast};
//! # let ast = procss::parse("div{.open{color:red;}}").unwrap();
//! use procss::RenderCss;
//!
//! let flat: ast::Css = ast.flatten_tree();
//! let css: String = flat.as_css_string();
//! assert_eq!(css, "div .open{color:red;}");
//! ```
//!
//! Intermediate structs [`ast::Css::transform`] amd [`ast::Tree::transform`]
//! can be used to recursively mutate a tree for a variety of node structs in
//! the [`ast`] module.  Some useful Example of such transforms can be
//! found in the [`transformers`] module.
//!
//! ```
//! # use procss::{parse, RenderCss};
//! use procss::transformers;
//!
//! let test = "
//! @mixin test {color: red;}
//! div {@include test;}
//! ";
//!
//! let mut ast = procss::parse(test).unwrap();
//! transformers::apply_mixin(&mut ast);
//! let flat = ast.flatten_tree().as_css_string();
//! assert_eq!(flat, "div{color:red;}");
//! ```
//!
//! For coordinating large builds on a tree of CSS files, the [`BuildCss`]
//! struct can parse and minify, applying all transforms (including
//! [`transformers::apply_import`]) as the compilation is left-folded over the
//! inputs.
//!
//! ```
//! let mut build = procss::BuildCss::new("./src");
//! build.add("controls/menu.scss");
//! build.add("logout.scss"); // imports "controls/menu.scss"
//! build.add("my_app.scss"); // imports "controls/menu.scss" and "logout.scss"
//! build.compile().unwrap().write("./dist").unwrap();
//! ```

#![feature(assert_matches)]
#![feature(path_file_prefix)]

pub mod ast;
mod parser;
mod render;
mod transform;
pub mod transformers;
mod utils;

use std::collections::HashMap;
#[cfg(not(feature = "iotest"))]
use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context};
use nom::error::convert_error;
use nom::Err;

use self::ast::Tree;
use self::parser::ParseCss;
pub use self::render::RenderCss;

/// Parse CSS text to a [`Tree`] (where it can be further manipulated),
/// capturing detailed error reporting for a moderate performance impact (using
/// [`nom::error::VerboseError`]).
///
/// # Example
///
/// ```rust
/// let ast = procss::parse("div { .open { color: red; }}").unwrap();
/// ```
pub fn parse(input: &str) -> anyhow::Result<Tree<'_>> {
    let (rest, x) = Tree::parse(input).map_err(|err| match err {
        Err::Error(e) | Err::Failure(e) => {
            anyhow!("Error parsing, unknown:\n{}", convert_error(input, e))
        }
        Err::Incomplete(needed) => anyhow!("Error parsing, unexpected input:\n {:?}", needed),
    })?;

    if rest.is_empty() {
        Ok(x)
    } else {
        Err(anyhow!("Error parsing, unreachable:\n {}", rest))
    }
}

/// Parse CSS text to a [`Tree`], without capturing error details, for maximum
/// performance without any error details when parsing fails.
///
/// # Example
///
/// ```rust
/// let ast = procss::parse_unchecked("div { .open { color: red; }}").unwrap();
/// ```
pub fn parse_unchecked(input: &str) -> anyhow::Result<Tree<'_>> {
    let (rest, x) = Tree::parse::<()>(input)?;
    if rest.is_empty() {
        Ok(x)
    } else {
        Err(anyhow!("Error parsing, unreachable:\n {}", rest))
    }
}

/// A CSS+ project build, comprising a collection of CSS+ files which may
/// reference eachother (via `@import`).
pub struct BuildCss<'a> {
    paths: Vec<String>,
    contents: HashMap<&'a str, String>,
    trees: HashMap<&'a str, ast::Tree<'a>>,
    css: HashMap<&'a str, ast::Css<'a>>,
    rootdir: &'static str,
}

/// The compiled output of a [`BuildCss`] collection, obtained from
/// [`BuildCss::compile`].  
pub struct CompiledCss<'a>(&'a BuildCss<'a>);

/// An incremental build struct for compiling a project's CSS sources.
///
/// # Example
///
/// ```
/// let mut build = procss::BuildCss::new("./src");
/// build.add("app.scss");
/// build.compile().unwrap().write("./dist").unwrap();
/// ```
impl<'a> BuildCss<'a> {
    /// Create a new [`BuildCss`] rooted at `rootdir`.
    pub fn new(rootdir: &'static str) -> Self {
        Self {
            paths: Default::default(),
            contents: Default::default(),
            trees: Default::default(),
            css: Default::default(),
            rootdir,
        }
    }

    /// Add a file `path` to this build.
    pub fn add(&mut self, path: &str) {
        self.paths.push(path.to_owned());
    }

    /// Compile this [`BuildCss`] start-to-finish, applying all transforms along
    /// the way.
    pub fn compile(&'a mut self) -> anyhow::Result<CompiledCss<'a>> {
        for path in &self.paths {
            let inpath = PathBuf::from(self.rootdir).join(path);
            let contents = fs::read_to_string(inpath)?;
            self.contents.insert(path, contents);
        }

        for (path, contents) in &self.contents {
            self.trees.insert(path, parse(contents)?);
        }

        let trees = self.trees.clone();
        for (path, tree) in self.trees.iter_mut() {
            transformers::apply_import(&trees)(tree);
            transformers::apply_mixin(tree);
            transformers::apply_var(tree);
            let mut flat = tree.flatten_tree();
            let outdir = utils::join_paths(self.rootdir, path);
            transformers::inline_url(&outdir.to_string_lossy())(&mut flat);
            transformers::dedupe(&mut flat);
            self.css.insert(path, flat);
        }

        Ok(CompiledCss(self))
    }
}

impl<'a> CompiledCss<'a> {
    /// Write this struct's compiled data to `outdir`, preserving the relative
    /// subdirectory structure of the `input` sources passed to
    /// [`BuildCss::add`], relative to `outdir`.
    pub fn write(self, outdir: &'static str) -> anyhow::Result<()> {
        for (path, css) in &self.0.css {
            let outpath = PathBuf::from(path);
            let outfile = format!(
                "{}.css",
                outpath
                    .file_prefix()
                    .context("No Prefix")?
                    .to_string_lossy()
            );

            let outdir = utils::join_paths(outdir, path);
            fs::create_dir_all(outdir.clone()).unwrap_or_default();
            fs::write(outdir.join(outfile), css.as_css_string())?;
        }

        Ok(())
    }
}

#[cfg(feature = "iotest")]
mod fs {
    //! In test mode, don't touch the file system.
    pub fn read_to_string<P: AsRef<std::path::Path>>(_path: P) -> std::io::Result<String> {
        Ok("div{}".to_owned())
    }

    pub fn write<P: AsRef<std::path::Path>, C: AsRef<[u8]>>(
        _path: P,
        _contents: C,
    ) -> std::io::Result<()> {
        Ok(())
    }

    pub fn create_dir_all<P: AsRef<std::path::Path>>(_path: P) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use super::*;

    #[test]
    fn test_verbose_error() {
        assert_matches!(
            parse("div{color:red").map(|x| x.as_css_string()).as_deref(),
            Err(_)
        )
    }

    #[test]
    fn test_parse_unchecked() {
        assert_matches!(
            parse_unchecked("div{color:red}")
                .map(|x| x.as_css_string())
                .as_deref(),
            Ok("div{color:red;}")
        )
    }
}

// `iotest` feature flag stubs out disk-accessing and other performance
// neutering function
#[cfg(all(not(feature = "iotest"), test))]
compile_error!("Feature 'iotest' must be enabled:\n\n> cargo test --features iotest\n\n");
