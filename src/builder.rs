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

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Context;

use crate::parser::{unwrap_parse_error, ParseCss};
use crate::render::RenderCss;
#[cfg(not(target_arch = "wasm32"))]
use crate::utils::fs;
#[cfg(feature = "iotest")]
use crate::utils::IoTestFs;
use crate::{ast, transformers, utils};

/// A CSS+ project build, comprising a collection of CSS+ files which may
/// reference eachother (via `@import`).
pub struct BuildCss<'a> {
    paths: Vec<String>,
    contents: HashMap<&'a str, String>,
    trees: HashMap<&'a str, ast::Tree<'a>>,
    css: HashMap<&'a str, ast::Css<'a>>,
    rootdir: String,
}

/// The compiled output of a [`BuildCss`] collection, obtained from
/// [`BuildCss::compile`].  
pub struct CompiledCss<'a>(&'a BuildCss<'a>);

/// An incremental build struct for compiling a project's CSS sources.
///
/// # Example
///
/// ```no_run
/// let mut build = procss::BuildCss::new("./src");
/// build.add_file("app.scss");
/// build.compile().unwrap().write("./dist").unwrap();
/// ```
impl<'a> BuildCss<'a> {
    /// Create a new [`BuildCss`] rooted at `rootdir`.
    pub fn new<S: Into<String>>(rootdir: S) -> Self {
        Self {
            paths: Default::default(),
            contents: Default::default(),
            trees: Default::default(),
            css: Default::default(),
            rootdir: rootdir.into(),
        }
    }

    /// Add a file `path` to this build.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn add_file(&mut self, path: &'a str) {
        self.paths.push(path.to_owned());
        let inpath = PathBuf::from(&self.rootdir).join(path);
        let txt = fs::read_to_string(inpath.as_path()).unwrap();
        self.contents.insert(path, txt);
    }

    /// Add a file `path` to this build.
    pub fn add_content(&mut self, path: &'a str, scss: String) {
        self.paths.push(path.to_owned());
        self.contents.insert(path, scss);
    }

    /// Compile this [`BuildCss`] start-to-finish, applying all transforms along
    /// the way.
    pub fn compile(&'a mut self) -> anyhow::Result<CompiledCss<'a>> {
        for (path, contents) in &self.contents {
            let tree = ast::Tree::parse(contents);
            let (_, tree) = tree.map_err(|err| unwrap_parse_error(contents, err))?;
            self.trees.insert(path, tree);
        }

        let dep_trees = self.trees.clone();
        for (path, tree) in self.trees.iter_mut() {
            transformers::apply_import(&dep_trees)(tree);
            transformers::apply_mixin(tree);
            transformers::apply_var(tree);
            self.css.insert(path, tree.flatten_tree());
        }

        for (path, css) in self.css.iter_mut() {
            let srcdir = utils::join_paths(&self.rootdir, path);
            transformers::inline_url(&srcdir.to_string_lossy())(css);
            transformers::dedupe(css);
        }

        Ok(CompiledCss(self))
    }
}

impl<'a> CompiledCss<'a> {
    /// Write this struct's compiled data to `outdir`, preserving the relative
    /// subdirectory structure of the `input` sources passed to
    /// [`BuildCss::add`], relative to `outdir`.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn write(self, outdir: &'static str) -> anyhow::Result<()> {
        for (outfile, css, path) in self.iter_files().flatten() {
            let outdir = utils::join_paths(outdir, path);
            fs::create_dir_all(outdir.clone()).unwrap_or_default();
            fs::write(outdir.join(outfile), css)?;
        }

        Ok(())
    }

    /// Render this struct's compiled data in memory as a `String`, preserving
    /// the relative subdirectory structure of the `input` sources passed to
    /// [`BuildCss::add`], relative to `outdir`.
    pub fn as_strings(&self) -> anyhow::Result<HashMap<String, String>> {
        let mut results = HashMap::default();
        for (outfile, css, _) in self.iter_files().flatten() {
            results.insert(outfile, css).unwrap_or_default();
        }

        Ok(results)
    }

    fn iter_files(&self) -> impl Iterator<Item = anyhow::Result<(String, String, &'_ str)>> {
        self.0.css.iter().map(|(path, css)| {
            let outpath = PathBuf::from(path);
            let outfile = format!(
                "{}.css",
                outpath
                    .file_prefix()
                    .context("No Prefix")?
                    .to_string_lossy()
            );

            Ok((outfile, css.as_css_string(), *path))
        })
    }
}

#[cfg(all(test, feature = "iotest"))]
mod tests {
    use std::cell::RefCell;
    use std::path::*;
    use std::rc::Rc;

    use super::*;

    #[test]
    fn test_simple_build() {
        let outputs = Rc::new(RefCell::new(vec![]));
        let infiles = Rc::new(RefCell::new(vec![]));
        let outfiles = Rc::new(RefCell::new(vec![]));

        let ctx = fs::read_to_string_context();
        let infiles2 = infiles.clone();
        ctx.expect().times(1).returning_st(move |x: &Path| {
            infiles2.borrow_mut().push(x.to_string_lossy().to_string());
            Ok("div{.open{color:green}}".to_owned())
        });

        let ctx = fs::create_dir_all_context();
        ctx.expect().times(1).returning(|_: PathBuf| Ok(()));

        let ctx = fs::write_context();
        let outputs2 = outputs.clone();
        let outfiles2 = outfiles.clone();
        ctx.expect().returning_st(move |x: PathBuf, y: String| {
            outfiles2.borrow_mut().push(x.to_string_lossy().to_string());
            outputs2.borrow_mut().push(y);
            Ok(())
        });

        let mut build = BuildCss::new("./src".to_owned());
        build.add_file("app/component.scss");
        build.compile().unwrap().write("./dist").unwrap();

        let outputs = outputs.borrow().clone();
        assert_eq!(outputs, vec!["div .open{color:green;}".to_owned()]);
        let infiles = infiles.borrow().clone();
        assert_eq!(infiles, vec!["./src/app/component.scss".to_owned()]);
        let outfiles = outfiles.borrow().clone();
        assert_eq!(outfiles, vec!["./dist/app/component.css".to_owned()])
    }
}
