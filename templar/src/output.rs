
use {Node, Element};
use std::io::{self, Write};

use escape::*;

pub type WriteResult<T> = Result<T, WriteError>;

pub enum WriteError {
    IO(io::Error),
    UnregocnizedDirective(String),
}

impl From<io::Error> for WriteError {
    fn from(err: io::Error) -> Self {
        WriteError::IO(err)
    }
}

pub fn write_out<W, F>(nodes:&[Node], writer:&mut W, indent: usize, f:&F) -> WriteResult<()> where W : Write, F : Fn(&str, &mut W) -> WriteResult<()> {
    for node in nodes {
        for _ in 0..indent {
            writer.write(b" ")?;
        }
        match node {
            &Node::Doctype(ref doctype) => {
                let out = format!("<!DOCTYPE {}>\n", doctype);
                writer.write(out.as_bytes())?;
            }
            &Node::Directive(ref directive) => {
                f(directive, writer)?;
            }
            &Node::Text(ref text) => {
                let out = escape_html(text).expect("escaped text");
                writer.write(out.as_bytes())?;
                writer.write(b"\n")?;
            },
            &Node::RawText(ref raw_text) => {
                writer.write(raw_text.as_bytes())?;
                writer.write(b"\n")?;
            },
            &Node::Element(ref element) => {
                let has_children = element.children.len() > 0;
                let trailing_slash : &str   = if has_children { "" } else { " /" };

                let open_tag : String = if element.attributes.is_empty() {
                    format!("<{}{}>", element.name, trailing_slash)
                } else {
                    let attributes : Vec<String> = element.attributes.iter().map(|&(ref k, ref v)|
                        format!("{}=\"{}\"", k, escape_default(v))
                    ).collect();
                    format!("<{} {}{}>", element.name, attributes.join(" "), trailing_slash)
                };
                writer.write(open_tag.as_bytes())?;
                writer.write(b"\n")?;
                if has_children {
                    // , |d| { f(d) }
                    write_out(element.children.as_slice(), writer, indent + 2, f);
                    let closing_tag : String = format!("</{}>", element.name);
                    for _ in 0..indent {
                        writer.write(b" ")?;
                    }
                    writer.write(closing_tag.as_bytes())?;
                    writer.write(b"\n")?;
                }
            },
        }
    }

    Ok(())
}
