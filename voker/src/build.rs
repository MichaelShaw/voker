
use sass_rs;
use std::path::{Path, PathBuf};
use std;
use std::fs;
use std::io;
use std::io::Write;
use std::io::Read;

use templar;


#[derive(Debug)]
pub struct ProcessedFile {
    pub source: PathBuf,
    pub action: BuildAction,
    pub result: Result<(), BuildErrorReason>,
}

#[derive(Debug)]
pub enum BuildAction {
    ScanDirectory,
    Copy(PathBuf),
    Ignore,
    Compile { extension: String, destination: PathBuf },
}

// build error should probably have some file params ... be a struct with a reason field
#[derive(Debug)]
pub enum BuildErrorReason {
    IO(io::Error),
    Sass(String),
    TemplarParse(templar::parse::ParseError),
    TemplarWrite(templar::output::WriteError<DirectiveError>),
    UTF8Error(std::string::FromUtf8Error),
}

impl From<io::Error> for BuildErrorReason {
    fn from(err: io::Error) -> Self {
        BuildErrorReason::IO(err)
    }
}

impl From<templar::output::WriteError<DirectiveError>> for BuildErrorReason {
    fn from(err: templar::output::WriteError<DirectiveError>) -> Self {
        BuildErrorReason::TemplarWrite(err)
    }
}

impl From<std::string::FromUtf8Error> for BuildErrorReason {
    fn from(err: std::string::FromUtf8Error) -> Self {
        BuildErrorReason::UTF8Error(err)
    }
}

impl From<templar::parse::ParseError> for BuildErrorReason {
    fn from(err: templar::parse::ParseError) -> Self {
        BuildErrorReason::TemplarParse(err)
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

pub fn build(source: &Path, destination: &Path) -> io::Result<Vec<ProcessedFile>> {
    fs::create_dir_all(destination)?;

    let paths = read_directory_paths(source)?;

    Ok(paths.into_iter().flat_map(|path| {
        if build_path(&path) {
            let new_dest = {
                let last = path.iter().last().expect("a last path component");
                destination.join(last)
            };

            if path.is_dir() {
                match build(&path, new_dest.as_path()) {
                    Ok(results) => results,
                    Err(io) => {
                        vec![ProcessedFile {
                            source: path,
                            action: BuildAction::ScanDirectory,
                            result: Err(BuildErrorReason::IO(io)),
                        }]
                    }
                }
            } else {
                let (action, result) : (BuildAction, Result<(), BuildErrorReason>) = match path.extension().and_then(|oss| oss.to_str()) {
                    Some("ace") => {
                        (
                            BuildAction::Compile { extension: "ace".into(), destination: new_dest.clone() },
                            compile_templar(source, &path, &new_dest)
                        )
                    },
                    Some("sass") => {
                        (
                            BuildAction::Compile { extension: "sass".into(), destination: new_dest.clone() },
                            compile_sass(&path, &new_dest)
                        )
                    },
                    _ => {
                        (
                            BuildAction::Copy(new_dest.clone()),
                            match fs::copy(&path, new_dest) {
                                Ok(_) => Ok(()),
                                Err(io) => Err(BuildErrorReason::IO(io)),
                            }
                        )
                    }
                };

                vec![ProcessedFile {
                    source: path,
                    action: action,
                    result: result,
                }]
            }
        } else {
            vec![ProcessedFile {
                source: path,
                action: BuildAction::Ignore,
                result: Ok(()),
            }]
        }
    }).collect())
}

pub fn compile_templar(base_directory:&Path, source:&Path, destination:&Path) -> Result<(), BuildErrorReason> {
    let directive_handler = TemplarDirectiveHandler { current_directory: base_directory.to_path_buf() };

    let nodes = parse_template(source)?;
    let out_path = destination.with_extension("html");
    let mut file = fs::File::create(out_path)?;

    templar::output::write_out(nodes.as_slice(), &mut file, 0, &directive_handler)?;
    file.sync_all()?;

    Ok(())
}

pub fn compile_sass(source:&Path, destination:&Path) -> Result<(), BuildErrorReason> {
    let out = sass_rs::compile_file(source, sass_rs::Options::default()).map_err(BuildErrorReason::Sass)?;
    write_to_path(&out, destination.with_extension("css").as_path())?;
    Ok(())
}

struct TemplarDirectiveHandler {
    pub current_directory: PathBuf,
}

#[derive(Debug)]
pub struct DirectiveError {
    pub directive: String,
    pub reason: String
}

impl templar::output::DirectiveHandler for TemplarDirectiveHandler {
    type DirectiveError = DirectiveError;

    fn handle<W>(&self, directive: &str, writer: &mut W) -> Result<(), DirectiveError> where W : Write {
        let parts : Vec<_> = directive.split(" ").collect();
        match parts.first() {
            Some(&"include") => {
                if let Some(second) = parts.get(1) {
                    let mut include_path = self.current_directory.clone();
                    include_path.push(second);
                    include_path.set_extension("ace");

                    let include_nodes = parse_template(&include_path).map_err(|e| {
                        DirectiveError {
                            directive: directive.to_string(),
                            reason: format!("{:?}", e)
                        }
                    })?;

                    templar::output::write_out(include_nodes.as_slice(), writer, 0, self).map_err(|e| {
                        DirectiveError {
                            directive: directive.to_string(),
                            reason: format!("{:?}", e)
                        }
                    })
                } else {
                    Err(DirectiveError {
                        directive: directive.to_string(),
                        reason: "unrecognized".to_string(),
                    })
                }
            },
            Some(&"doctype") => {
                writer.write_all(b"<!DOCTYPE html>").map_err(|_| DirectiveError {
                    directive: directive.to_string(),
                    reason: "couldnt write doctype".to_string(),
                })
            },
            _ => {
                Err(DirectiveError {
                    directive: directive.to_string(),
                    reason: "unrecognized".to_string(),
                })
            }
        }
    }
}


pub fn parse_template(path:&Path) -> Result<Vec<templar::Node>, BuildErrorReason> {
    let template_str = read_path(&path)?;
    let template_nodes = templar::parse::parse(&template_str)?;
    Ok(template_nodes)
}

pub fn read_path(path:&Path) -> Result<String, BuildErrorReason> {
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

pub fn read_directory_paths(path:&Path) -> io::Result<Vec<PathBuf>> {
    let mut paths : Vec<PathBuf> = Vec::new();

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let file_path = entry.path().to_path_buf();
        paths.push(file_path);
    }

    Ok(paths)
}