extern crate df2;
extern crate docopt;
extern crate rustc_serialize;

use std::path::Path;

use df2::{Reader, Shot};
use docopt::Docopt;

const USAGE: &'static str = "
Query Optech df2 files.

Usage:
    df2 summary <path>

Options:
    -h --help       Show this screen.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    cmd_summary: bool,
    arg_path: String,
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());
    if args.cmd_summary {
        print_summary(args.arg_path);
    }
}

fn print_summary<P: AsRef<Path>>(path: P) {
    println!("Filename: {}", path.as_ref().to_string_lossy());
    let shots = Reader::from_path(path)
        .and_then(|reader| reader.collect::<Result<Vec<Shot>, _>>())
        .unwrap();
    println!("Number of shots: {}", shots.len());
}
