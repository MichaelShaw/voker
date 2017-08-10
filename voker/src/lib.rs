extern crate templar;
extern crate sass_rs;

#[macro_use]
extern crate serde_derive;
extern crate docopt;
extern crate colored;
extern crate notify;

extern crate hyper;
extern crate futures;
extern crate futures_cpupool;

extern crate mime_guess;

pub mod build;
pub mod command;
pub mod server;
pub mod watch;
pub mod build_feedback;


use std::io::Write;


pub struct DirectivePrinter {}

impl templar::output::DirectiveHandler for DirectivePrinter {
    type DirectiveError = String;
    fn handle<W>(&self, directive: &str, _: &mut W) -> Result<(), Self::DirectiveError> where W : Write {
        println!("handle directive -> {:?}", directive);
        Ok(())
    }
}