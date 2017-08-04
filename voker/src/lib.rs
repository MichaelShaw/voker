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
            for node in &nodes {
                print_ele(node, 0);
            }
        }
        Err(e) => {
            println!("parse error -> {:?}", e);
        }
    }

}

pub fn print_ele(node:&Node, indent: usize) {
    for _ in 0..indent {
        print!(" ");
    }
    match node {
        &Node::Doctype(ref doctype) => {
            println!("<!DOCTYPE {}>", doctype);
        }
        &Node::Directive(ref directive) => {
            println!("= {}", directive);
        }
        &Node::Text(ref text) => {
            println!("{}", text);
        },
        &Node::Element(ref element) => {
            if element.attributes.is_empty() {
                println!("<{}>", element.name);
            } else {
                let attributes : Vec<String> = element.attributes.iter().map(|&(ref k, ref v)|
                    format!("{}=\"{}\"", k, v)
                ).collect();
                println!("<{} {}>", element.name, attributes.join(" "));
            }
            for node in &element.children {
                print_ele(node, indent + 2);
            }
        },
    }
}