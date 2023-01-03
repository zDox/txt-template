use templ_parse::parse_str;

use std::io::{Read, self};
use std::env;

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

    let tokens = parse_str(&content).unwrap();
    let display = env::var("DISPLAY").unwrap();
    if &display == "true" {
        println!("Input: \"{}\"\n", content);
        println!("Output: \"{:#?}\"", tokens);
    } else {
        print!(".");
    }
}

