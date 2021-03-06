
use sass_rs;
use std::path::{Path, PathBuf};
use std;
use std::fs;
use std::io;
use std::io::{Write, Read};
use filetime::{FileTime, set_file_times};


use templar;
use templar::{TemplateContext, Node};

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
    Skip, // no change
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
                    Some("tlr") => {
                        (
                            BuildAction::Compile { extension: "tlr".into(), destination: new_dest.clone() },
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
                        if same_attributes(&path, &new_dest) {
                            (BuildAction::Skip, Ok(()))
                        } else {
                            (
                                BuildAction::Copy(new_dest.clone()),
                                match copy_maintaining_modified_time(&path, &new_dest) {
                                    Ok(_) => Ok(()),
                                    Err(io) => Err(BuildErrorReason::IO(io)),
                                }
                            )
                        }
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

pub fn copy_maintaining_modified_time(source:&Path, dest:&Path) -> io::Result<()> {
    fs::copy(source, dest).and_then(|_|
        fs::metadata(source)
    ).and_then(|metadata| {
        let mtime = FileTime::from_last_modification_time(&metadata);
        let atime = FileTime::from_last_access_time(&metadata);
        set_file_times(dest, atime, mtime)
    })
}

pub fn same_attributes(a: &Path, b:&Path) -> bool {
    if a.exists() && b.exists() {
        let a_meta = a.metadata();
        let b_meta = b.metadata();
        if let (Some(a_md), Some(b_md)) = (a_meta.ok(), b_meta.ok()) {
            let a_time = FileTime::from_last_modification_time(&a_md);
            let b_time = FileTime::from_last_modification_time(&b_md);
            a_time == b_time && a_md.len() == b_md.len()
        } else {
            false
        }
    } else {
        false
    }
}

pub fn compile_templar(base_directory:&Path, source:&Path, destination:&Path) -> Result<(), BuildErrorReason> {
    let directive_handler = TemplarDirectiveHandler { current_directory: base_directory.to_path_buf() };

    let nodes = parse_template(source)?;
    let out_path = destination.with_extension("html");
    let mut file = fs::File::create(out_path)?;

    let empty_context = TemplateContext::empty();

    templar::output::write_out(nodes.as_slice(), &empty_context, &mut file, 0, 2, &directive_handler)?;
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

    fn handle<W>(&self, context:&TemplateContext, command: &str, children: &[Node], base_indent:usize, indent_size: usize, writer: &mut W) -> Result<(), DirectiveError> where W : Write {
        let parts : Vec<_> = command.split(" ").collect();
        match parts.first() {
            Some(&"yield") => {
                templar::output::write_out(context.nodes.as_slice(), &context, writer, base_indent, indent_size, self).map_err(|e| {
                    DirectiveError {
                        directive: command.to_string(),
                        reason: format!("{:?}", e)
                    }
                })
            },
            Some(&"include") => {
                if let Some(second) = parts.get(1) {
                    let mut include_path = self.current_directory.clone();
                    include_path.push(second);
                    include_path.set_extension("tlr");

                    let include_nodes = parse_template(&include_path).map_err(|e| {
                        DirectiveError {
                            directive: command.to_string(),
                            reason: format!("{:?}", e)
                        }
                    })?;

                    let context = TemplateContext {
                        nodes: children.iter().cloned().collect(),
                    };

                    templar::output::write_out(include_nodes.as_slice(), &context, writer, base_indent, indent_size, self).map_err(|e| {
                        DirectiveError {
                            directive: command.to_string(),
                            reason: format!("{:?}", e)
                        }
                    })
                } else {
                    Err(DirectiveError {
                        directive: command.to_string(),
                        reason: "unrecognized".to_string(),
                    })
                }
            },
            Some(&"doctype") => {
                writer.write_all(b"<!DOCTYPE html>").map_err(|_| DirectiveError {
                    directive: command.to_string(),
                    reason: "couldnt write doctype".to_string(),
                })
            },
            _ => {
                Err(DirectiveError {
                    directive: command.to_string(),
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