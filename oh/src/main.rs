extern crate clap;
extern crate data_encoding;
extern crate sha2;
extern crate walkdir;

mod cli;
mod config;
mod objects;
mod sinks;

use sinks::ObjectSink;
use walkdir::WalkDir;

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map_or(false, |s| s != ".git" && s.starts_with('.'))
}

fn walk_dir(dir: &::std::path::Path, sink: &mut sinks::ObjectSink, app: &config::OHApp) {
    let root = match dir.canonicalize() {
        Ok(path) => path,
        Err(_) => return,
    };
    let mut iter = WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !is_hidden(e));
    loop {
        let entry = match iter.next() {
            None => break,
            Some(Ok(entry)) => entry,
            Some(Err(_)) => {
                continue;
            }
        };

        if app.exclude_paths.contains(entry.path()) {
            iter.skip_current_dir();
        } else if entry.file_type().is_dir()
            && entry
                .path()
                .file_name()
                .unwrap_or(&std::ffi::OsStr::new(""))
                == ".git"
        {
            iter.skip_current_dir();
            objects::git::walk_repo(entry.path().parent().unwrap(), sink);
        } else {
            sink.push(&objects::file::SimpleFile {
                path: std::path::PathBuf::from(entry.path()),
            });
        }
    }
}

fn main() {
    let matches = cli::get_app().get_matches();

    let app_config = config::OHApp::new(&std::path::PathBuf::from("/home/lachlan/.oh")).unwrap();
    let mut db_sink = sinks::LMDBSink::new(&app_config).expect("Failed to open database.");

    if let Some(sc_matches) = matches.subcommand_matches("scan") {
        match sc_matches.value_of("DIRECTORY") {
            Some(dir) => {
                walk_dir(&std::path::PathBuf::from(dir), &mut db_sink, &app_config);
            }
            None => {
                for path in &app_config.search_paths {
                    walk_dir(&path, &mut db_sink, &app_config);
                }
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("find") {
        let search_hash = data_encoding::HEXLOWER_PERMISSIVE
            .decode(matches.value_of("HASH").unwrap().as_bytes())
            .expect("Bad hash format.");
        match db_sink.lookup(&search_hash) {
            Ok(result) => println!("{}", result),
            Err(err) => {
                println!("Error: {}", err);
                std::process::exit(1);
            }
        }
    }
}
