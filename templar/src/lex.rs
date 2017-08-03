use nom;
use nom::*; // {digit, space, alphanumeric}
use std::str;


// tokens
#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    Indentation(u32),
    Element(String),
    ElementId(String),
    ElementClass(String),
    Text(String),
    Directive(Vec<String>),
}

pub type LexResult = Result<Vec<Vec<Token>>, LexError>;

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

// captures indentation length in characters (for tracking nesting)
named!(indentation<usize>,
    map_res!(
        space,
        character_length
    )
);

named!(line_p<Option<usize>>,
    opt!(indentation)
);

named!(identifier<&str>,
    map_res!(
        alphanumeric,
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
        rr: rest >>
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

pub fn lex(content:&str) -> LexResult {

    println!("lexing content length: {:?} ", content.len());

    let lines = content.lines().enumerate();

    for (line_idx, line) in lines {
        println!("{:?}: in -> {:?}", line_idx, line);
        let line_result = line_p(line.as_bytes());
        println!("{:?}: out -> {:?}", line_idx, line_result);

    }




    // step 1. break in to seperate parts

    Ok(Vec::new())
}