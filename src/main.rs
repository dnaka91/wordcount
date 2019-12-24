//! # Wordcount Coding Challenge 2019
//!
//! The challenge is to scan a large file of text for the occurrence of words and count these. The
//! goal is to do this as fast as possible and with a low usage of RAM.
//!
//! Words come from a text file containing 1000 words, each separated by a newline (`\n`). The list
//! can contain duplicates. For example:
//!
//! ```txt
//! cat
//! cat
//! dog
//! it doesn't have.
//! proprietary
//! ...
//! ...
//! ```
//!
//! The text file to scan contains 1 GB of Wikipedia articles.
//!
//! ## Output format
//!
//! The program has to output the counts for each word in the order the words were present in the
//! input file. Therefore, the output should als be exactly 1000 lines long.
//!
//! ```txt
//! 1000
//! 1000
//! 2000
//! 123
//! 22
//! ...
//! ...
//! ```
//!
//! ## Further details
//!
//! - Text can be overlapping, for example `textext` results in 2 counts for `text`, one for
//! **text**ext and one for tex**text**.
//! - The words and article are considered case-sensitive. `Text` and `text` are not the same.
//! - All input should be treated as ASCII text.
//!

#![forbid(unsafe_code)]
#![deny(clippy::all, clippy::pedantic, clippy::nursery, clippy::cargo)]

use std::env;
use std::error::Error;
use std::fs;
use std::io::BufReader;
use std::sync::atomic::AtomicU64;

use getopts::Options;
use indexmap::map::IndexMap;

mod aho_corasick;
mod ahocorasick;
mod fourtytwo;
mod naive;

const OPT_NAIVE: &str = "naive";
const OPT_FOURTYTWO: &str = "fourtytwo";
const OPT_HELP: &str = "help";
const OPT_VERSION: &str = "version";

fn main() -> Result<(), Box<dyn Error>> {
    // First parsing CLI input.
    let mut opts = Options::new();
    opts.optflag("n", OPT_NAIVE, "Use a naive custom implementation");
    opts.optflag("f", OPT_FOURTYTWO, "Use the fastest algorithm in the world");
    opts.optflag("h", OPT_HELP, "Print this help menu");
    opts.optflag("v", OPT_VERSION, "Show the program version");

    let matches = opts.parse(env::args().skip(1))?;

    if matches.opt_present(OPT_VERSION) {
        print_version();
        return Ok(());
    }

    if matches.opt_present(OPT_HELP) || matches.free.len() != 2 {
        print_usage(&opts);
        return Ok(());
    }

    // Then we need to open our input files for processing.
    let words = String::from_utf8(fs::read(&matches.free[0])?)?;
    let words: Vec<_> = words.split_terminator('\n').collect();
    let words_map: IndexMap<_, _> = words.iter().map(|s| (*s, AtomicU64::default())).collect();

    let article = fs::File::open(&matches.free[1])?;
    let article = BufReader::new(article);

    // Here is the core logic for counting words. Everything else is just preparation
    // like parsing CLI options, opening the files and so on.
    if matches.opt_present(OPT_NAIVE) {
        naive::process(&words_map, article);
    } else if matches.opt_present(OPT_FOURTYTWO) {
        fourtytwo::process(&words_map, article);
    } else {
        ahocorasick::process(&words_map, article);
    }

    // Printing out our findings.
    for w in words {
        println!("{:?}", words_map[w]);
    }

    Ok(())
}

/// Print out instructions about how to use this program.
fn print_usage(opts: &Options) {
    let brief = format!("Usage: {} WORDS_FILE ARTICLE_FILE [options]", env!("CARGO_PKG_NAME"));
    print!("{}", opts.usage(&brief));
}

/// Print out the program's version number.
fn print_version() {
    println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
}
