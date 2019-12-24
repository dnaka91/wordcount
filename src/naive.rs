//! Solving the challenge with a custom but naive and slow algorithm.

use std::io::{BufRead, BufReader, Read};
use std::sync::atomic::{AtomicU64, Ordering};

use indexmap::map::IndexMap;
use rayon::prelude::*;

#[allow(dead_code, unused_variables)]
pub(crate) fn process<R>(words: &IndexMap<&str, AtomicU64>, article: BufReader<R>)
where
    R: Read + Send,
{
    let patterns: Vec<_> = words.keys().collect();

    article.lines().par_bridge().filter_map(Result::ok).for_each(|line| {
        for p in &patterns {
            let mut r = &line[..];

            while let Some(i) = r.find(*p) {
                words[*p].fetch_add(1, Ordering::SeqCst);
                r = &r[i + 1..];
            }
        }
    });
}
