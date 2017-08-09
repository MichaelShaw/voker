
use sass_rs;
use std::path::{Path, PathBuf};
use std;
use std::fs;
use std::io;
use std::io::Write;
use std::io::Read;

use templar;

pub type BuildResult<T> = Result<T, BuildError>;


// build error should probably have some file params ... be a struct with a reason field
#[derive(Debug)]
pub enum BuildError {
    IO(io::Error),
    Sass(String),
    TemplarParse(templar::parse::ParseError),
    TemplarWrite(templar::output::WriteError<String>),
    UTF8Error(std::string::FromUtf8Error),
}

impl From<io::Error> for BuildError {
    fn from(err: io::Error) -> Self {
        BuildError::IO(err)
    }
}

impl From<templar::output::WriteError<String>> for BuildError {
    fn from(err: templar::output::WriteError<String>) -> Self {
        BuildError::TemplarWrite(err)
    }
}

impl From<std::string::FromUtf8Error> for BuildError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        BuildError::UTF8Error(err)
    }
}

impl From<templar::parse::ParseError> for BuildError {
    fn from(err: templar::parse::ParseError) -> Self {
        BuildError::TemplarParse(err)
    }
}

pub fn build_path(path:&Path) -> bool {
    let path = path.iter().last().expect("a last component in a path");
    if let Some(path_str) = path.to_str() {
        !(path_str.starts_with(".") || path_str.starts_with("_"))
    } else {
        false
    }
}

pub fn build(source: &Path, destination: &Path) -> BuildResult<()> {
    println!("building {:?} -> {:?}", source, destination);

    fs::create_dir_all(destination)?;

    let paths = read_directory_paths(source)?;

    for path in paths {
        if build_path(&path) {
            let last = path.iter().last().expect("a last path component");
            let new_dest = destination.join(last);

            if path.is_dir() {
                println!("found directory -> {:?}", path);
                build(&path, new_dest.as_path())?;
            } else {
                match path.extension().and_then(|oss| oss.to_str()) {
                    Some("ace") => {
                        println!("found an ace -> {:?}", path);
                        let directive_handler = TemplarDirectiveHandler { current_directory: source.to_path_buf() };

                        let nodes = parse_template(&path)?;
                        let out_path = new_dest.with_extension("html");
                        let mut file = fs::File::create(out_path)?;

                        templar::output::write_out(nodes.as_slice(), &mut file, 0, &directive_handler)?;
                        file.sync_all()?;
                    },
                    Some("sass") => {
                        println!("found a sass -> {:?}", path);
                        let out = sass_rs::compile_file(&path, sass_rs::Options::default()).map_err(BuildError::Sass)?;
                        write_to_path(&out, new_dest.with_extension("css").as_path())?;
                    },
                    _ => {
                        println!("copying -> {:?}", path);
                        fs::copy(&path, new_dest)?;
                    }
                }
            }
        } else {
            println!("ignoring {:?}", path);
        }
    }

    Ok(())
}

struct TemplarDirectiveHandler {
    pub current_directory: PathBuf,
}

impl templar::output::DirectiveHandler for TemplarDirectiveHandler {
    type DirectiveError = String;

    fn handle<W>(&self, directive: &str, writer: &mut W) -> Result<(), String> where W : Write {
        println!("handle directive -> {:?}", directive);
        let parts : Vec<_> = directive.split(" ").collect();
        match parts.first() {
            Some(&"include") => {
                if let Some(second) = parts.get(1) {
                    let mut include_path = self.current_directory.clone();
                    include_path.push(second);
                    include_path.set_extension("ace");
                    println!("include -> {:?}", include_path);
                    let include_nodes = parse_template(&include_path).map_err(|e| format!("{:?}", e))?;

                    templar::output::write_out(include_nodes.as_slice(), writer, 0, self).map_err(|e| format!("{:?}", e))?;

                    Ok(())
                } else {
                    Err(format!("unrecognized directive -> {:?}", directive))
                }
            },
            Some(&"doctype") => {
                writer.write_all(b"<!DOCTYPE html>").map_err(|e|String::from("couldnt write"))
            },
            _ => Err(format!("unrecognized directive -> {:?}", directive))
        }
    }
}


pub fn parse_template(path:&Path) -> BuildResult<Vec<templar::Node>> {
    let template_str = read_path(&path)?;
    let template_nodes = templar::parse::parse(&template_str)?;
    Ok(template_nodes)
}

pub fn read_path(path:&Path) -> BuildResult<String> {
    let mut f = fs::File::open(path)?;
    let mut bytes = Vec::new();
    f.read_to_end(&mut bytes)?;
    let s = String::from_utf8(bytes)?;
    Ok(s)
}

pub fn write_to_path(str:&str, path:&Path) -> io::Result<()> {
    let mut file = fs::File::create(path)?;
    file.write_all(str.as_bytes())?;
    Ok(())
}

pub fn read_directory_paths(path:&Path) -> BuildResult<Vec<PathBuf>> {
    let mut paths : Vec<PathBuf> = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path().to_path_buf();
        paths.push(file_path);
    }

    Ok(paths)
}