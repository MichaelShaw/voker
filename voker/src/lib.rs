extern crate templar;
extern crate sass_rs;

#[macro_use]
extern crate serde_derive;
extern crate docopt;

extern crate notify;

extern crate hyper;
extern crate futures;
extern crate futures_cpupool;

extern crate mime_guess;

pub mod build;
pub mod command;
pub mod server;
pub mod watch;


use std::fs;
use std::io::Write;
use std::io::Read;

use templar::{Node, Element};

pub fn run_samples() {
    let mut f = fs::File::open("resources/_head.ace").expect(" a file");

    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes).expect("some bytes");

    let str = std::str::from_utf8(&bytes).expect(" a string");

    let parse_result = templar::parse::parse(str);

    match parse_result {
        Ok(nodes) => {
            println!("parse result!");
            let directive_handler = |directive:&str, out: &mut std::io::Stdout| {
                println!("directive handler -> {:?}", directive);
                let result : Result<(), String> = Ok(());
                result
            };
            templar::output::write_out(nodes.as_slice(), &mut std::io::stdout() , 0, &DirectivePrinter {});
        }
        Err(e) => {
            println!("parse error -> {:?}", e);
        }
    }
}

pub struct DirectivePrinter {}

impl templar::output::DirectiveHandler for DirectivePrinter {
    type DirectiveError = String;
    fn handle<W>(&self, directive: &str, writer: &mut W) -> Result<(), Self::DirectiveError> where W : Write {
        println!("handle directive -> {:?}", directive);
        Ok(())
    }
}