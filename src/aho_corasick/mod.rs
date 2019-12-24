#![allow(clippy::similar_names, clippy::module_name_repetitions, clippy::cast_possible_truncation)]

pub use ahocorasick::{AhoCorasick, FindIter};

mod ahocorasick;
mod automaton;
mod dfa;
mod nfa;
mod state_id;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Match {
    pattern: usize,
    len: usize,
    end: usize,
}

impl Match {
    #[inline]
    pub const fn pattern(&self) -> usize {
        self.pattern
    }

    #[inline]
    pub const fn end(&self) -> usize {
        self.end
    }
}
