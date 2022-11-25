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
use crate::utils::fs;
#[cfg(feature = "iotest")]
use crate::utils::IoTestFs;
use crate::{ast, transformers, utils};

/// A CSS+ project build, comprising a collection of CSS+ files which may
/// reference eachother (via `@import`).
pub struct BuildCss {
    paths: Vec<String>,
    rootdir: &'static str,
}

/// An incremental build struct for compiling a project's CSS sources.
///
/// # Example
///
/// ```no_run
/// let mut build = procss::BuildCss::new("./src");
/// build.add("app.scss");
/// build.compile("./dist").unwrap();
/// ```
impl BuildCss {
    /// Create a new [`BuildCss`] rooted at `rootdir`.
    pub fn new(rootdir: &'static str) -> Self {
        Self {
            paths: Default::default(),
            rootdir,
        }
    }

    /// Add a file `path` to this build.
    pub fn add(&mut self, path: &str) {
        self.paths.push(path.to_owned());
    }

    /// Compile this [`BuildCss`] start-to-finish, applying all transforms along
    /// the way. Resulting files are written to `outdir`, preserving their paths
    /// relative to `rootdir` used to construct this [`BuildCss`].
    pub fn compile(&self, outdir: &'static str) -> anyhow::Result<()> {
        let mut contents: HashMap<&str, String> = HashMap::default();
        let mut trees: HashMap<&str, ast::Tree> = HashMap::default();
        let mut css: HashMap<&str, ast::Css> = HashMap::default();
        for path in &self.paths {
            let inpath = PathBuf::from(self.rootdir).join(path);
            let txt = fs::read_to_string(inpath.as_path())?;
            contents.insert(path, txt);
        }

        for (path, contents) in &contents {
            let tree = ast::Tree::parse(contents);
            let (_, tree) = tree.map_err(|err| unwrap_parse_error(contents, err))?;
            trees.insert(path, tree);
        }

        let dep_trees = trees.clone();
        for (path, tree) in trees.iter_mut() {
            transformers::apply_import(&dep_trees)(tree);
            transformers::apply_mixin(tree);
            transformers::apply_var(tree);
            css.insert(path, tree.flatten_tree());
        }

        for (path, css) in css.iter_mut() {
            let outdir = utils::join_paths(self.rootdir, path);
            transformers::inline_url(&outdir.to_string_lossy())(css);
            transformers::dedupe(css);
        }

        for (path, css) in css {
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

        let mut build = BuildCss::new("./src");
        build.add("app/component.scss");
        build.compile("./dist").unwrap();

        let outputs = outputs.borrow().clone();
        assert_eq!(outputs, vec!["div .open{color:green;}".to_owned()]);
        let infiles = infiles.borrow().clone();
        assert_eq!(infiles, vec!["./src/app/component.scss".to_owned()]);
        let outfiles = outfiles.borrow().clone();
        assert_eq!(outfiles, vec!["./dist/app/component.css".to_owned()])
    }
}
