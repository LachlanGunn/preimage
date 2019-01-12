extern crate config;

#[derive(Debug)]
pub enum ConfigError {
    PathExists,
}

#[derive(Debug, Clone)]
pub struct PreimageApp {
    pub path: ::std::path::PathBuf,
    pub search_paths: ::std::vec::Vec<::std::path::PathBuf>,
    pub exclude_paths: ::std::collections::BTreeSet<::std::path::PathBuf>,
}

impl ::std::fmt::Display for ConfigError {
    fn fmt(&self, fmt: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
        write!(
            fmt,
            "Preimage Config Error: {}",
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

fn get_default_config() -> Result<std::vec::Vec<u8>,Box<std::error::Error>> {
    let home = std::env::var("HOME")?;
    Ok(format!("search-paths:
  - {}
exclude-paths:
  - {}/bin", home, home).into_bytes())
}

fn get_or_create_data_dir(
    path: &::std::path::Path,
) -> Result<::std::path::PathBuf,Box<std::error::Error>> {
    use std::io::Write;

    if path.is_dir() {
        return Ok(::std::path::PathBuf::from(path));
    }

    if path.exists() {
        return Err(Box::from(ConfigError::PathExists));
    }

    ::std::fs::create_dir(path)?;
    ::std::fs::create_dir(path.join(::std::path::PathBuf::from("db")))?;
    let mut file = ::std::fs::File::create(path.join(::std::path::PathBuf::from("preimage.yaml")))?;
    file.write_all(&get_default_config()?)?;
    Ok(::std::path::PathBuf::from(path))
}

impl PreimageApp {
    pub fn new(path: &::std::path::Path) -> Result<PreimageApp, Box<::std::error::Error>> {
        let data_dir = get_or_create_data_dir(path)?;

        let mut settings = config::Config::default();
        settings.merge(config::File::from(
            path.join(::std::path::PathBuf::from("preimage.yaml")),
        ))?;

        Ok(PreimageApp {
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
