use nom::*; // {digit, space, alphanumeric}
use std::str;
use colored::Colorize;

fn always<T>(_:T) -> bool {
    true
}

fn is_identifier(b:u8) -> bool {
    let c = b as char;
    c.is_alphanumeric() || c == '-' || c == '_'
}

fn noneify_blank_string(str: &str) -> Option<String> {
    if str.is_empty() {
        None
    }  else {
        Some(str.into())
    }
}

// captures indentation length in characters (for tracking nesting)
// need to handle zero indentation
named!(identifier<&str>,
    map_res!(
        take_while1!(is_identifier),
        str::from_utf8
    )
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

named!(attribute_content<&str>,
    map_res!(
        take_till!(is_space),
        str::from_utf8
    )
);

named!(key_value_pair<(String, String)>,
    do_parse!(
        k: identifier >>
        tag!("=") >>
        v: alt_complete!( attribute_content | quoted_value ) >>
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

named!(doctype_line<LineContent>,
    do_parse!(
        tag!("doctype") >>
        rr: rest >>
        ( LineContent::Doctype(rr.to_string()) )
    )
);

named!(javascript_line<LineContent>,
    do_parse!(
        tag!(":javascript") >>
        rr : rest >>
        ( LineContent::Javascript )
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
                classes_ids: class_ids,
                attributes: kvps,
                inner_text: noneify_blank_string(rr.trim()),
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
                classes_ids: class_ids,
                attributes: kvps,
                inner_text: noneify_blank_string(rr.trim()),
          })
        )
    )
);

named!(line_p<LineContent>,
    alt_complete!(doctype_line | javascript_line | tag_element_line | class_id_only_line | directive_line | text_line | empty_line)
);

#[derive(Debug)]
enum ClassId {
    Id(String),
    Class(String),
}

#[derive(Debug)]
struct Element {
    tag: Option<String>,
    classes_ids: Vec<ClassId>,
    attributes: Vec<(String, String)>,
    inner_text: Option<String>,
}

#[derive(Debug)]
enum LineContent {
    Javascript,
    Doctype(String),
    Element(Element),
    Directive(String),
    Text(String),
    Empty,
}

#[derive(Debug)]
struct Line {
    pub indentation: usize,
    pub content: LineContent,
}

fn indentation(str: &str) -> Option<usize> {
    str.chars().position(|c| !c.is_whitespace())
}

pub type LexResult = Result<Vec<super::Element>, LexError>;

#[derive(Debug)]
pub struct LexError {
    pub line: u64,
    pub character: Option<u64>,
    pub kind: Option<u32>,
}


fn c_ok(c:char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_'
}

named!(str_test<&str, &str>,
    take_while1!(c_ok)
);

fn t_str() {
    let res = str_test("defini_tels_f dent");
    println!("res -> {:?}", res);

}

pub fn lex(content:&str) -> LexResult {
    t_str();
    return Ok(Vec::new());

    let mut out = Vec::new();

    for (line_idx, line) in content.lines().enumerate() {
        // indentation and slicing first
        println!("line -> {:?}", line);
        if let Some(indent) = indentation(line) {
            let (_, rest) = line.split_at(indent);
            let line_content_result = line_p(rest.as_bytes());

            match line_content_result {
                IResult::Done(i, o) => {
                    println!("Done-> {}", format!("{:?}",o).green())
                },
                IResult::Error(err) => {
//                    err = 12;

                    println!("Err -> {}", format!("{:?}", err).red())
                },
                IResult::Incomplete(needed) => {

                    println!("Incomplete -> {}", format!("{:?}", needed).yellow())
                },
            }
        } else {
            out.push(Line { indentation : 0, content: LineContent::Empty });
        }
    }

    Ok(Vec::new())
}