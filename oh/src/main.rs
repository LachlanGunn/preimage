extern crate clap;
extern crate data_encoding;
extern crate futures;
extern crate sha2;
extern crate tokio;
extern crate walkdir;

mod cli;
mod config;
mod objects;
mod sinks;
mod sources;

use crate::sinks::ObjectSink;

fn main() {
    let matches = cli::get_app().get_matches();

    let app_config = config::OHApp::new(&std::path::PathBuf::from("/home/lachlan/.oh")).unwrap();

    if let Some(sc_matches) = matches.subcommand_matches("scan") {
        match sc_matches.value_of("DIRECTORY") {
            Some(dir) => {
                let mut sink = sinks::LMDBSink::new(&app_config).expect("Failed to open database.");
                sources::walk_dir(&std::path::PathBuf::from(dir), sink, &app_config);
            }
            None => {
                for path in &app_config.search_paths {
                    let mut sink =
                        sinks::LMDBSink::new(&app_config).expect("Failed to open database.");
                    sources::walk_dir(&path, sink, &app_config);
                }
            }
        }
    } else if let Some(matches) = matches.subcommand_matches("find") {
        let search_hash = data_encoding::HEXLOWER_PERMISSIVE
            .decode(matches.value_of("HASH").unwrap().as_bytes())
            .expect("Bad hash format.");
        let mut sink = sinks::LMDBSink::new(&app_config).expect("Failed to open database.");
        match sink.lookup(&search_hash) {
            Ok(result) => println!("{}", result),
            Err(err) => {
                println!("Error: {}", err);
                std::process::exit(1);
            }
        }
    }
}
