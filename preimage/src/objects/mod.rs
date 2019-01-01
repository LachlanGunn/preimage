pub mod file;

#[cfg(feature = "git")]
pub mod git;

pub trait Object {
    fn hash(&self) -> Result<::std::vec::Vec<::std::vec::Vec<u8>>, Box<::std::error::Error>>;
    fn to_str(&self) -> String;
}
