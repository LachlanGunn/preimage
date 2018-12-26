extern crate data_encoding;
extern crate git2;
extern crate sha2;

pub struct GitCommit {
    pub path: ::std::path::PathBuf,
    pub id: git2::Oid,
}

impl crate::objects::Object for GitCommit {
    fn hash(&self) -> Result<::std::vec::Vec<::std::vec::Vec<u8>>, Box<::std::error::Error>> {
        Ok(vec![::std::vec::Vec::from(self.id.as_bytes())])
    }

    fn to_str(&self) -> String {
        format!("git:{}", self.path.to_str().unwrap())
    }
}

pub struct GitFile {
    pub commit: GitCommit,
    pub path: String,
    pub hash: std::vec::Vec<u8>,
    pub id: std::vec::Vec<u8>,
}

impl crate::objects::Object for GitFile {
    fn hash(&self) -> Result<::std::vec::Vec<::std::vec::Vec<u8>>, Box<::std::error::Error>> {
        Ok(vec![
            ::std::vec::Vec::from(&self.id[..]),
            ::std::vec::Vec::from(&self.hash[..]),
        ])
    }

    fn to_str(&self) -> String {
        format!(
            "git-file:{}:{}:{}",
            self.commit.path.to_string_lossy(),
            data_encoding::HEXLOWER.encode(self.commit.id.as_bytes()),
            self.path
        )
    }
}
