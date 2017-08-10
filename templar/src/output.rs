
use {Node, Element};
use std::io::{self, Write};

use escape::*;

#[derive(Debug)]
pub enum WriteError<DE> {
    DirectiveError(DE),
    IO(io::Error),
}

impl<DE> From<io::Error> for WriteError<DE> {
    fn from(err: io::Error) -> Self {
        WriteError::IO(err)
    }
}

pub trait DirectiveHandler {
    type DirectiveError;
    fn handle<W>(&self, directive: &str, writer: &mut W) -> Result<(), Self::DirectiveError> where W : Write;
}

pub fn write_out<W, DH>(nodes:&[Node], writer:&mut W, indent: usize, directive_handler:&DH) -> Result<(), WriteError<DH::DirectiveError>>
    where W : Write, DH: DirectiveHandler {
    for node in nodes {
//        for _ in 0..indent {
//            writer.write(b" ")?;
//        }
        match node {
            &Node::Doctype(ref doctype) => {
                let out = format!("<!DOCTYPE {}>\n", doctype);
                writer.write(out.as_bytes())?;
            }
            &Node::Directive(ref directive) => {
                directive_handler.handle(directive, writer).map_err(WriteError::DirectiveError)?;
            }
            &Node::Text(ref text) => {
//                let out = escape_html(text).expect("escaped text");
//                writer.write(out.as_bytes())?;
                writer.write(text.as_bytes())?;
//                writer.write(b"\n")?;
            },
            &Node::RawText(ref raw_text) => {
                writer.write(raw_text.as_bytes())?;
                writer.write(b"\n")?;
            },
            &Node::Element(ref element) => {
                let seperate_close_tag = element.children.len() > 0 || element.name == "script" || element.name == "a";
                let trailing_slash : &str = if !seperate_close_tag { " /" } else { " " };

//                println!("ele -> {:?} Close tag -> {:?} trailing slash -> {:?}", element, seperate_close_tag, trailing_slash);

                let open_tag : String = if element.attributes.is_empty() {
                    format!("<{}{}>", element.name, trailing_slash)
                } else {
                    let attributes : Vec<String> = element.attributes.iter().map(|&(ref k, ref v)|
                        format!("{}=\"{}\"", k, escape_default(v))
                    ).collect();
                    format!("<{} {}{}>", element.name, attributes.join(" "), trailing_slash)
                };
                writer.write(open_tag.as_bytes())?;
                if seperate_close_tag {
                    write_out(element.children.as_slice(), writer, indent + 2, directive_handler)?;
                    let closing_tag : String = format!("</{}>", element.name);
//                    for _ in 0..indent {
//                        writer.write(b" ")?;
//                    }
                    writer.write(closing_tag.as_bytes())?;
//                    writer.write(b"\n")?;
                }
            },
        }
    }

    Ok(())
}
