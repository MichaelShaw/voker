use nom;
use nom::*; // {digit, space, alphanumeric}
use std::str;


pub type LexResult = Result<Vec<Line>, LexError>;

#[derive(Debug)]
pub struct LexError {
    pub line: u64,
    pub character: u64,
}

fn always<T>(t:T) -> bool {
    true
}

fn id<T>(t:T) -> T {
    t
}

fn character_length(bytes: &[u8]) -> Result<usize, str::Utf8Error> {
    let st = str::from_utf8(bytes)?;
    Ok(st.len())
}

fn to_str(v:Vec<char>) -> String {
    v.into_iter().collect()
}

fn is_identifier(b:u8) -> bool {
    let c = b as char;
    c.is_alphanumeric() || c == '-'
}

// captures indentation length in characters (for tracking nesting)
// need to handle zero indentation
named!(indentation<usize>,
    map_res!(
        take_while!(is_space),
        character_length
    )
);

named!(identifier<&str>,
    map_res!(
        take_while1!(is_identifier),
        str::from_utf8
    )
);

named!(element<&str>,
    map!(identifier, id)
);

named!(element_id<&str>,
    do_parse!(
        tag!("#") >>
        id : identifier >>
        (id)
    )
);

named!(rest<&str>,
    map_res!(take_while!(always), str::from_utf8)
);

named!(element_class<&str>,
    do_parse!(
        tag!(".") >>
        class_name : identifier >>
        (class_name)
    )
);

named!(quoted_value<&str>,
    map_res!(
        delimited!(
            tag!("\""),
            take_until!("\""),
            tag!("\"")
        ),
        str::from_utf8
    )
);

named!(key_value_pair<(String, String)>,
    do_parse!(
        k: identifier >>
        tag!("=") >>
        v: alt!( identifier | quoted_value ) >>
        (k.into(), v.into())
    )
);

named!(text_line<LineContent>,
    do_parse!(
        tag!("|") >>
        rr: map!(rest, |s| s.trim() ) >>
        ( LineContent::Text(rr.to_string()) )
    )
);

named!(directive_line<LineContent>,
    do_parse!(
        tag!("=") >>
        rr : rest >>
        ( LineContent::Directive(rr.trim().to_string()) )
    )
);

named!(empty_line<LineContent>,
    do_parse!(
//        many0!(space) >>
        eof!() >>
        ( LineContent::Empty )
    )
);

named!(tag_element_line<LineContent>,
    do_parse!(
        tag: identifier >>
        class_ids: many0!(
            alt!(
                map!(element_class, |s| ClassId::Class(s.to_string())) |
                map!(element_id, |s| ClassId::Id(s.to_string()))
            )
        ) >>
        many0!(space) >>
        kvps: many0!(ws!(key_value_pair)) >>
        many0!(space) >>
        rr: rest >>
        ( LineContent::Element(Element {
                tag: Some(tag.to_string()),
                stuff: class_ids,
                attributes: kvps,
                inner_text: Some(rr.into()),
          })
        )
    )
);

named!(line_p<Line>,
    do_parse!(
        indent: indentation >>
        line_content: alt_complete!(tag_element_line | directive_line | text_line | empty_line) >>

        ( Line { indentation: indent, content: line_content } )
    )
);

#[derive(Debug)]
pub enum ClassId {
    Id(String),
    Class(String),
}

#[derive(Debug)]
pub struct Element {
    tag: Option<String>,
    stuff: Vec<ClassId>,
    attributes: Vec<(String, String)>,
    inner_text: Option<String>,
}

#[derive(Debug)]
pub enum LineContent {
    Element(Element),
    Directive(String),
    Text(String),
    Empty,
}

#[derive(Debug)]
pub struct Line {
    pub indentation: usize,
    pub content: LineContent,
}

pub fn lex(content:&str) -> LexResult {

    println!("lexing content length: {:?} ", content.len());

    let lines = content.lines().enumerate();

    for (line_idx, line) in lines {
        println!("{:?}: in -> {:?}", line_idx, line);
        let line_result = line_p(line.as_bytes());
        match line_result {
            IResult::Done(i, o) => println!("Done-> {:?}", o),
            IResult::Error(err) => println!("Err -> {:?}", err),
            IResult::Incomplete(needed) => println!("Incomplete -> {:?}", needed),
        }


    }




    // step 1. break in to seperate parts

    Ok(Vec::new())
}