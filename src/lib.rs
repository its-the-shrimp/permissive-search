#![no_std]

extern crate alloc;

pub mod lookalikes;

use {
    alloc::{string::String, vec, vec::Vec},
    core::mem::swap,
};

/// A tree that associates a string key with an `usize` index.
///
/// The tree structure allows for a more optimised search that allows suspending the search, e.g.
/// while user input is pending.
#[derive(Debug, Default)]
pub struct SearchTree {
    nodes: Vec<(char, SearchTree)>,
    end: Option<usize>,
}

impl<'key> FromIterator<(usize, &'key str)> for SearchTree {
    fn from_iter<T: IntoIterator<Item = (usize, &'key str)>>(iter: T) -> Self {
        let mut res = Self::default();
        for (index, key) in iter {
            res.push(key, index);
        }
        res
    }
}

impl SearchTree {
    /// Get an immediate child node associated with the provided character.
    pub fn get(&self, index: char) -> Option<&Self> {
        if self.nodes.last().is_none_or(|(last, _)| index > *last) {
            return None;
        }

        self.nodes
            .binary_search_by_key(&index, |(ch, _)| *ch)
            .ok()
            .map(|i| &self.nodes[i].1)
    }

    /// Add a key to the tree
    pub fn push(&mut self, key: &str, index: usize) {
        let mut iter = key.chars();
        let Some(ch) = iter.next() else {
            self.end = Some(index);
            return;
        };

        let i = match self.nodes.binary_search_by_key(&ch, |(ch, _)| *ch) {
            Ok(i) => i,
            Err(i) => {
                self.nodes.insert(i, (ch, Self::default()));
                i
            }
        };

        self.nodes[i].1.push(iter.as_str(), index);
    }

    fn for_each_base<E>(&self, f: &mut impl FnMut(usize) -> Result<(), E>) -> Result<(), E> {
        self.end.map(&mut *f).transpose()?;
        self.nodes
            .iter()
            .try_for_each(|(_, node)| node.for_each_base(f))
    }

    /// Calls a function on all the keys reachable from this tree node.
    ///
    /// # Errors
    /// The function doesn't fail itself, but it does propagate errors from the callback
    pub fn for_each<E>(&self, mut f: impl FnMut(usize) -> Result<(), E>) -> Result<(), E> {
        self.for_each_base(&mut f)
    }
}

pub struct Searcher<'tree> {
    root: &'tree SearchTree,
    input: String,
    /// Nodes in consideration
    considered: Vec<&'tree SearchTree>,
    /// To be swapped with `considered` after every char input
    new: Vec<&'tree SearchTree>,
}

impl Extend<char> for Searcher<'_> {
    fn extend<T: IntoIterator<Item = char>>(&mut self, iter: T) {
        for ch in iter {
            self.push(ch);
        }
    }
}

impl<'tree> Searcher<'tree> {
    /// Create a searcher through a tree with `root` as its root.
    pub fn new(root: &'tree SearchTree) -> Self {
        Self {
            root,
            input: String::new(),
            considered: vec![root],
            new: vec![],
        }
    }

    pub const fn root(&self) -> &'tree SearchTree {
        self.root
    }

    pub fn input(&self) -> &str {
        self.input.as_str()
    }

    /// Push a character into the searched string
    pub fn push(&mut self, ch: char) {
        self.input.push(ch);
        Self::compute_considerations(&mut self.considered, &mut self.new, ch);
    }

    /// Common impl for [`Searcher::push`] & [`Searcher::pop`]
    fn compute_considerations(
        considered: &mut Vec<&'tree SearchTree>,
        new: &mut Vec<&'tree SearchTree>,
        ch: char,
    ) {
        let misclicks = lookalikes::all(ch);
        new.clear();
        new.extend(
            considered.iter().filter_map(|n| n.get(ch)).chain(
                considered
                    .iter()
                    .flat_map(|n| misclicks.clone().filter_map(|ch| n.get(ch))),
            ),
        );

        if !new.is_empty() {
            swap(new, considered);
        }
    }

    /// Remove the last character from the searched string, if present.
    pub fn pop(&mut self) {
        if self.input.pop().is_none() {
            return;
        }
        self.considered.clear();
        self.considered.push(self.root);
        for ch in self.input.chars() {
            Self::compute_considerations(&mut self.considered, &mut self.new, ch);
        }
    }

    /// Calls a function on every key that could've been referred to by the current input.
    ///
    /// # Errors
    /// The function doesn't fail itself, but it does propagate errors from the callback
    pub fn for_each_candidate<E>(
        &self,
        mut f: impl FnMut(usize) -> Result<(), E>,
    ) -> Result<(), E> {
        self.considered
            .iter()
            .try_for_each(|n| n.for_each_base(&mut f))
    }
}
