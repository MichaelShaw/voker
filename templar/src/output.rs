
use {Node, Element};
use std::io::{self, Write};

use escape::*;

pub fn write_out<W>(nodes:&[Node], writer:&mut W, indent: usize) -> io::Result<()> where W : Write {
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
                let out = format!("<!--{}-->\n", directive);
                writer.write(out.as_bytes())?;
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
                    write_out(element.children.as_slice(), writer, indent + 2);
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
//
//
//pub fn print_ele(node:&Node, indent: usize) {
//    for _ in 0..indent {
//        print!(" ");
//    }

//}