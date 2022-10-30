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

use crate::render::RenderCss;

/// A wrapper around [`Vec`] which guarantees at least `N` elements.
#[derive(Clone, Debug)]
pub struct MinVec<T, const N: usize>([T; N], Vec<T>);

impl<T, const N: usize> MinVec<T, N>
where
    T: std::fmt::Debug,
{
    /// Create a new N-element-guaranteed collection.
    pub fn new(head: [T; N], tail: Vec<T>) -> Self {
        MinVec(head, tail)
    }

    /// Iterate over the values in this collection.
    pub fn iter(&self) -> impl Iterator<Item = &'_ T> {
        self.0.iter().chain(self.1.iter())
    }

    /// Iterate over the values in this collection.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &'_ mut T> {
        self.0.iter_mut().chain(self.1.iter_mut())
    }
}

impl<T: RenderCss, const N: usize> RenderCss for MinVec<T, N> {
    fn render(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for x in self.0.iter() {
            x.render(f)?;
        }

        for x in self.1.iter() {
            write!(f, ",")?;
            x.render(f)?;
        }

        Ok(())
    }
}
