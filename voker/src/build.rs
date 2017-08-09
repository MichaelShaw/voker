
use sass_rs;
use std::path::{Path, PathBuf};
use std::fs;
use std::io;
use std::io::Write;

pub type BuildResult<T> = Result<T, BuildError>;

#[derive(Debug)]
pub enum BuildError {
    IO(io::Error),
    Sass(String),
}

impl From<io::Error> for BuildError {
    fn from(err: io::Error) -> Self {
        BuildError::IO(err)
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