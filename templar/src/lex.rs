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
        space,
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

named!(text<&str>,
    do_parse!(
        tag!("|") >>
        rr: map!(rest, |s| s.trim() ) >>
        (rr)
    )
);

named!(element_class<&str>,
    do_parse!(
        tag!(".") >>
        class_name : identifier >>
        (class_name)
    )
);

named!(directive<&str>,
    do_parse!(
        tag!("=") >>
        rr : rest >>
        (rr)
    )
);

named!(text_line<LineContent>,
    map!(
        text,
        |s| LineContent::Text(s.to_string())
    )
);

named!(directive_line<LineContent>,
    map!(
        directive,
        |s| LineContent::Directive(s.to_string())
    )
);

named!(empty_line<LineContent>,
    do_parse!(
        many0!(space) >>
        eof!() >>
        ( LineContent::Empty )
    )
);

named!(line_p<Line>,
    do_parse!(
        indent: opt!(indentation) >>
        line_content: alt!(text_line | directive_line | empty_line) >>

        ( Line { indentation: indent.unwrap_or(12), content: line_content } )
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
    inner_text: Option<String>,
}

#[derive(Debug)]
pub enum LineContent {
    Element(String),
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