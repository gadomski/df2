extern crate df2;
extern crate docopt;
extern crate rustc_serialize;

use df2::{Reader, Shot};
use docopt::Docopt;
use rustc_serialize::json;
use std::path::Path;

const USAGE: &'static str = "
Query Optech df2 files.

Usage:
    df2 shot <path> <number>
    df2 summary <path>

Options:
    -h --help       Show this screen.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    cmd_shot: bool,
    cmd_summary: bool,
    arg_path: String,
    arg_number: u16,
}

fn main() {
    let args: Args = Docopt::new(USAGE).and_then(|d| d.decode()).unwrap_or_else(|e| e.exit());
    if args.cmd_shot {
        print_shot(args.arg_path, args.arg_number);
    } else if args.cmd_summary {
        print_summary(args.arg_path);
    }
}
fn print_shot<P: AsRef<Path>>(path: P, number: u16) {
    let mut reader = Reader::from_path(path).unwrap();
    reader.seek(number).unwrap();
    let shot = reader.read_one().unwrap().unwrap();
    println!("{}", json::as_json(&shot));
}

fn print_summary<P: AsRef<Path>>(path: P) {
    println!("Filename: {}", path.as_ref().to_string_lossy());
    let shots = Reader::from_path(path)
        .and_then(|reader| reader.collect::<Result<Vec<Shot>, _>>())
        .unwrap();
    println!("Number of shots: {}", shots.len());
}
