extern crate sha2;

use sha2::Digest;
use tokio::prelude::*;

const BLOCK_SIZE: usize = 4096;
type HashFuture = Future<
    Item = future::Loop<std::vec::Vec<std::vec::Vec<u8>>, (tokio::fs::File, sha2::Sha256)>,
    Error = tokio::io::Error,
>;

pub struct SimpleFile {
    pub path: ::std::path::PathBuf,
}

fn hash_file(state: (tokio::fs::File, sha2::Sha256)) -> Box<HashFuture> {
    let (fh, mut hasher) = state;
    let buf = vec![0u8; BLOCK_SIZE];
    Box::from(tokio::io::read(fh, buf).and_then(move |(fh, buf, count)| {
        hasher.input(&buf[0..count]);

        if count == BLOCK_SIZE {
            futures::future::ok(future::Loop::Continue((fh, hasher)))
        } else {
            //let hasher = std::rc::try_unwrap(hasher_rc);
            futures::future::ok(future::Loop::Break(vec![std::vec::Vec::from(
                &hasher.result()[..],
            )]))
        }
    }))
}

impl crate::objects::Object for SimpleFile {
    fn hash(&self) -> Result<::std::vec::Vec<::std::vec::Vec<u8>>, Box<::std::error::Error>> {
        let file_future = tokio::fs::File::open(self.path.to_path_buf());

        let hasher = sha2::Sha256::default();
        file_future
            .and_then(|fh| future::loop_fn((fh, hasher), &hash_file))
            .or_else(|e| Err(Box::from(e)))
            .wait()
    }

    fn to_str(&self) -> String {
        format!("file:{}", self.path.to_str().unwrap())
    }
}
