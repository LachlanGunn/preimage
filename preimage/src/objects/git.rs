extern crate data_encoding;
extern crate git2;

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
