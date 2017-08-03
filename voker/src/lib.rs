extern crate templar;
extern crate sass_rs;

use std::fs;
use std::io::Read;

pub fn run_samples() {
    let mut f = fs::File::open("resources/pages/index.ace").expect(" a file");

    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes).expect("some bytes");

    let str = std::str::from_utf8(&bytes).expect(" a string");

    let lex_result = templar::lex::lex(str);



    println!("lex result -> {:?}", lex_result);

//    templar::lex::lex()
//    println!("Hello, world!");


//    let out = sass_rs::compile_file("resources/main.sass", sass_rs::Options::default());

//    println!("out -> {:?}", out);


}
