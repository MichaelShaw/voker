#[macro_use]
extern crate nom;
extern crate colored;

pub mod lex;
pub mod escape;


#[derive(Debug, Clone)]
pub struct Element {
    name: String,
    attributes: Vec<(String, String)>,
    children: Vec<Node>,
}

#[derive(Debug, Clone)]
pub enum Node {
    Directive(String),
    Text(String),
    Element(Element),
}