
use std::env;
use std::io;
use docopt::Docopt;
use build;
use server;
use watch;
use std::thread;

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

pub fn run_docopt() -> io::Result<()> {
    let args: Args = Docopt::new(USAGE)
        .map(|d| d.version(Some("0.1".into())))
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let current_directory = env::current_dir()?;

    println!("current dir -> {:?}", current_directory);

    if args.cmd_serve {
        if let Some(ref name) = args.arg_name {
            let mut source = current_directory.clone();
            source.push(name);
            let mut dest = current_directory.clone();
            dest.push("_out");
            dest.push(name);
            println!("serve ... building -> {:?} @ {:?}", source, dest);

            let server_root = dest.clone();
            let server_join_handle = thread::spawn(|| {
                let server_config = server::ServerConfig {
                    addr: "127.0.0.1:3000".parse().unwrap(),
                    root_dir: server_root,
                    num_file_threads: 4,
                    num_server_threads: 4,
                };
                server::serve(server_config);
            });

            let build_result = build::build(&source, &dest);
            println!("initial build result -> {:?}", build_result);
            let watcher = watch::watch(&source);
            'fs: loop {
                match watcher.change_events.recv() {
                    Ok(watch::ChangeEvent{ path, op:_, cookie:_ }) => {
                        if let Some(p) = path {
                            let build_result = build::build(&source, &dest);
                            println!("initial build result -> {:?}", build_result);
                        }
                    },
                    Err(_) => break 'fs,
                }
            }

            // serve name
        } else {
            // serve all
        }

    } else if args.cmd_build {
        println!("build");
        if let Some(ref name) = args.arg_name {
            // build name
            let mut source = current_directory.clone();
            source.push(name);
            let mut dest = current_directory.clone();
            dest.push("_out");
            dest.push(name);
            let build_result = build::build(&source, &dest);
            println!("build result -> {:?}", build_result);

        } else {
            // build all
        }

    } else {
        println!("WTTTF");
    }

    Ok(())
}