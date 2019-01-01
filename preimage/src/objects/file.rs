extern crate sha2;

use std::io::Read;

use sha2::Digest;

pub struct SimpleFile {
    pub path: ::std::path::PathBuf,
}

impl crate::objects::Object for SimpleFile {
    fn hash(&self) -> Result<::std::vec::Vec<::std::vec::Vec<u8>>, Box<::std::error::Error>> {
        let mut fh = ::std::fs::File::open(&self.path)?;

        const BLOCK_SIZE: usize = 4096;
        let mut buf = [0u8; BLOCK_SIZE];
        let mut hasher = sha2::Sha256::default();
        loop {
            let count = fh.read(&mut buf)?;
            hasher.input(&buf[0..count]);
            if count < BLOCK_SIZE {
                break;
            }
        }

        Ok(vec![::std::vec::Vec::from(&hasher.result()[..])])
    }

    fn to_str(&self) -> String {
        format!("file:{}", self.path.to_str().unwrap())
    }
}
