extern crate clap;

pub fn get_app() -> clap::App<'static, 'static> {
    clap::App::new("preimage")
        .version("0.1")
        .author("Lachlan Gunn <lachlan@gunn.ee>")
        .about("Locate data by its hash.")
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            clap::SubCommand::with_name("scan")
                .about("Update the location database")
                .arg(
                    clap::Arg::with_name("DIRECTORY")
                        .help("Override the paths in the configuration file.")
                        .required(false)
                        .index(1),
                ),
        )
        .subcommand(
            clap::SubCommand::with_name("find")
                .about("Find the location of the object represented by <HASH>")
                .arg(
                    clap::Arg::with_name("HASH")
                        .help("Hash to look up.")
                        .required(true)
                        .index(1),
                ),
        )
}
