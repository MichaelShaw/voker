use nom;
use nom::*; // {digit, space, alphanumeric}
use std::str;
use colored::Colorize;

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

fn noneify_blank_string(str: &str) -> Option<String> {
    if str.is_empty() {
        None
    }  else {
        Some(str.into())
    }
}

fn noneify_blank_string_opt(str: Option<&str>) -> Option<String> {
    match str {
        Some(s) =>  {
            if s.is_empty() {
                None
            } else {
                Some(s.into())
            }
        },
        None => None,
    }
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


named!(mah_content<&str>,
    map_res!(
        take_till!(is_space),
        str::from_utf8
    )
);

named!(key_value_pair<(String, String)>,
    do_parse!(
        k: identifier >>
        tag!("=") >>
        v: alt_complete!( identifier | quoted_value ) >>
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
        eof!() >>
        ( LineContent::Empty )
    )
);

named!(tag_element_line<LineContent>,
    do_parse!(
        tag: identifier >>
        class_ids: many0!(
            alt_complete!(
                map!(element_class, |s| ClassId::Class(s.to_string())) |
                map!(element_id, |s| ClassId::Id(s.to_string()))
            )
        ) >>
        kvps: many0!(ws!(complete!(key_value_pair))) >>
        rr : rest >>
        ( LineContent::Element(Element {
                tag: Some(tag.to_string()),
                stuff: class_ids,
                attributes: kvps,
                inner_text: noneify_blank_string(rr),
//                stuff: Vec::new(),
//                attributes: Vec::new(),
//                inner_text: None,
          })
        )
    )
);

named!(class_id_only_line<LineContent>,
    do_parse!(
        class_ids: many1!(
            alt_complete!(
                map!(element_class, |s| ClassId::Class(s.to_string())) |
                map!(element_id, |s| ClassId::Id(s.to_string()))
            )
        ) >>
        kvps: many0!(ws!(complete!(key_value_pair))) >>
        rr : rest >>
        ( LineContent::Element(Element {
                tag: None,
                stuff: class_ids,
                attributes: kvps,
                inner_text: noneify_blank_string(rr),
          })
        )
    )
);

named!(line_p<Line>,
    do_parse!(
        indent: indentation >>
        line_content: alt_complete!(tag_element_line | class_id_only_line | directive_line | text_line | empty_line) >>
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
//    let r = tag_element_line("a.whatever key=hoopl jack=pot".as_bytes());
//    println!("res -> {:?}", r);
//
//    return Ok(Vec::new());

    println!("lexing content length: {:?} ", content.len());

    let lines = content.lines().enumerate();

    for (line_idx, line) in lines {

        let line_result = line_p(line.as_bytes());
        println!("{}: in -> {}", line_idx, line);
        match line_result {
            IResult::Done(i, o) => println!("Done-> {}", format!("{:?}",o).green()),
            IResult::Error(err) => {

                println!("Err -> {}", format!("{:?}", err).red())
            },
            IResult::Incomplete(needed) => {
//                println!("{}: in -> {}", line_idx, line);
                println!("Incomplete -> {}", format!("{:?}", needed).yellow())
            },
            _ => (),
        }


    }




    // step 1. break in to seperate parts

    Ok(Vec::new())
}