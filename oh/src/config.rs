extern crate config;

#[derive(Debug)]
pub enum ConfigError {
    PathExists,
}

#[derive(Debug, Clone)]
pub struct OHApp {
    pub path: ::std::path::PathBuf,
    pub search_paths: ::std::vec::Vec<::std::path::PathBuf>,
    pub exclude_paths: ::std::collections::BTreeSet<::std::path::PathBuf>,
}

impl ::std::fmt::Display for ConfigError {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(
            fmt,
            "OH Config Error: {}",
            match self {
                ConfigError::PathExists => "Path already exists.",
            }
        )
    }
}

impl ::std::error::Error for ConfigError {
    fn cause(&self) -> Option<&::std::error::Error> {
        None
    }
}

fn get_or_create_data_dir(
    path: &::std::path::Path,
) -> Result<::std::path::PathBuf, Box<::std::error::Error>> {
    use std::io::Write;

    if path.is_dir() {
        return Ok(::std::path::PathBuf::from(path));
    }

    if path.exists() {
        return Err(Box::new(ConfigError::PathExists));
    }

    ::std::fs::create_dir(path)?;
    ::std::fs::create_dir(path.join(::std::path::PathBuf::from("db")))?;
    let mut file = ::std::fs::File::create(path.join(::std::path::PathBuf::from("oh.yaml")))?;
    file.write_all(
        b"search-paths:\n    - /home/lachlan/\nexclude-paths:\n    - /home/lachlan/bin",
    )?;
    Ok(::std::path::PathBuf::from(path))
}

impl OHApp {
    pub fn new(path: &::std::path::Path) -> Result<OHApp, Box<::std::error::Error>> {
        let data_dir = get_or_create_data_dir(path)?;

        let mut settings = config::Config::default();
        settings.merge(config::File::from(
            path.join(::std::path::PathBuf::from("oh.yaml")),
        ))?;

        Ok(OHApp {
            path: data_dir,
            search_paths: settings
                .get_array("search-paths")?
                .into_iter()
                .map(|p: config::Value| ::std::path::PathBuf::from(p.into_str().unwrap()))
                .collect::<::std::vec::Vec<_>>(),
            exclude_paths: settings
                .get_array("exclude-paths")?
                .into_iter()
                .map(|p: config::Value| ::std::path::PathBuf::from(p.into_str().unwrap()))
                .collect::<::std::collections::BTreeSet<_>>(),
        })
    }
}
