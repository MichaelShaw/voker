

use std::io;
use build::*;
use std::path::Path;
use colored::Colorize;

//ScanDirectory,
//Copy(PathBuf),
//Ignore,
//Compile { extension: String, destination: PathBuf },

pub fn print_summary(path:&Path, result: io::Result<Vec<ProcessedFile>>) {
    let l = format!("\n\n\nBuilding {:?}\n", path);
    println!("{}", l.cyan());

    match result {
        Ok(files) => {
            for file in files {
                let color = match file.action {
                    BuildAction::Ignore => "yellow",
                    _ => if file.result.is_ok() { "green" } else { "red" }
                };
                let line = format!("{:?} - {:?} - {:?}", file.source, file.action, file.result );
                println!("{}", line.color(color));
            }
        }
        Err(io_error) => {
            let line = format!("io error -> {:?}", io_error);
            println!("{}", line.red());
        }
    }
}