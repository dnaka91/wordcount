//! Solving the challenge with [Aho Corasick][aho_corasick_wiki].
//!
//! [aho_corasick_wiki]: https://en.wikipedia.org/wiki/Aho%E2%80%93Corasick_algorithm

use std::io::{BufRead, BufReader, Read, Result};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::aho_corasick::AhoCorasick;
use indexmap::map::IndexMap;
use rayon::prelude::*;

/// The maximum lines to chunk together into a single string. This value showed the best results for
/// the input given during the challenge.
const LINE_LIMIT: u64 = 10000;

pub fn process<R>(words: &IndexMap<&str, AtomicU64>, article: BufReader<R>)
where
    R: Read + Send,
{
    let patterns: Vec<_> = words.keys().collect();

    // Prepare the automaton.
    let ac = AhoCorasick::new(&patterns);

    // Run the automaton on every line separately on multiple
    // threads to improve throughput.
    chunked_lines(article, LINE_LIMIT).par_bridge().map(Result::unwrap).for_each(|line| {
        for mat in ac.find_overlapping_iter(&line) {
            words.get_index(mat.pattern()).unwrap().1.fetch_add(1, Ordering::SeqCst);
        }
    });
}

struct ChunkedLines<B> {
    buf: B,
    limit: u64,
}

/// Similar to [`lines`] returns an iterator over the lines of a reader, but instead of
/// iterating over each line they are chunked together into a single string until the `limit` or EOL
/// is reached.
///
/// Also,  in contrast to [`lines`] each string returned *will contain* the newline bytes and CRLFs.
///
/// [`lines`]: https://doc.rust-lang.org/std/io/trait.BufRead.html#method.lines
const fn chunked_lines<R>(buf: R, limit: u64) -> ChunkedLines<R>
where
    R: BufRead + Sized,
{
    ChunkedLines { buf, limit }
}

impl<B: BufRead> Iterator for ChunkedLines<B> {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = String::new();
        let mut lines = 0;

        loop {
            match self.buf.read_line(&mut buf) {
                // Finish on EOL. If the buffer has remaining data,
                // return it first and finish in the next call.
                Ok(0) => {
                    if buf.is_empty() {
                        return None;
                    }
                    break;
                }
                // Collect lines until we reach the limit.
                Ok(_) => {
                    lines += 1;
                    if lines > self.limit {
                        break;
                    }
                }
                // Report back any errors of read calls.
                Err(e) => return Some(Err(e)),
            }
        }

        // Either we hit the line limit or reached the end and have
        // remaining data in the buffer.
        Some(Ok(buf))
    }
}
