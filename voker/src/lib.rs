extern crate templar;
extern crate sass_rs;

use std::fs;
use std::io::Read;

use templar::{Node, Element};

pub fn run_samples() {
    let mut f = fs::File::open("resources/pages/index.ace").expect(" a file");

    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes).expect("some bytes");

    let str = std::str::from_utf8(&bytes).expect(" a string");

    let parse_result = templar::parse::parse(str);

    match parse_result {
        Ok(nodes) => {
            println!("parse result!");
            templar::output::write_out(nodes.as_slice(), &mut std::io::stdout() , 0);
        }
        Err(e) => {
            println!("parse error -> {:?}", e);
        }
    }

}
