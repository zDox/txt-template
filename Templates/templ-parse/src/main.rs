use templ_parse::parse_str;
use std::fs::File;
use std::io::Read;
use std::env;

fn main() {
    let mut args = env::args();
    args.next();
    let filename = args.next().expect("Missing filename");
    let mut file = File::open(filename).unwrap();

    let content = {
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        buf
    };    

    let tokens = parse_str(&content).unwrap();
    println!("Input: {}\n", content);
    println!("Output: {:#?}\n", tokens);
}

