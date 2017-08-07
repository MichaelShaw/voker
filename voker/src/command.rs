

use docopt::Docopt;

const USAGE: &'static str = "
Voker Static Site Gen

Usage:
  voker serve
  voker serve <name>
  voker build
  voker build <name>
  voker (-h | --help)
  voker --version

Options:
  -h --help     Show this screen.
  --version     Show version.
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_name: Option<String>,
    cmd_serve: bool,
    cmd_build: bool,
}

pub fn run_docopt() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    println!("{:?}", args);
}