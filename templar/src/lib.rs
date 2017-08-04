#[macro_use]
extern crate nom;
extern crate colored;


pub mod parse;
pub mod escape;
pub mod output;


#[derive(Debug, Clone)]
pub struct Element {
    pub name: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone)]
pub enum Node {
    Doctype(String),
    Directive(String),
    Text(String),
    RawText(String), // for javascript
    Element(Element),
}

pub fn element(name:&str, attributes: Vec<(&str, &str)>) -> Element {
    Element {
        name: name.into(),
        attributes: attributes.iter().map(|&(k, v)| (k.into(), v.into())).collect(),
        children: Vec::new(),
    }
}

pub fn contains<T, F>(opt: Option<T>, f: F) -> bool where F: Fn(&T) -> bool {
    opt.iter().any(f)
}
