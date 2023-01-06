use templ_parse::parse_str;

use std::io::{Read, self};
use std::env;
use std::process;

fn main() {
    let stdin = io::stdin();
    let mut stdin = stdin.lock();
    let mut line = String::new();
    let mut content = String::new();

    while let Ok(n_bytes) = stdin.read_to_string(&mut line) {
        if n_bytes == 0 { break }
        content.push_str(&line);
        line.clear();
    }

    let tokens = match parse_str(&content) {
        Ok(tokens) => tokens,
        Err(err) => {
            dbg!(&err);
            eprintln!("{}", err);
            process::exit(1);
        },
    };

    let mut args = env::args();
    args.next();
    if let Some(arg) = args.next() {
        if arg == "--display" {
            println!("Input: \"{}\"\n", content);
            println!("Output: \"{:#?}\"", tokens);
        } else {
            println!("Unknown argument: Know arguments are --display");
        }
    } else {
        print!(".");
    }
}

