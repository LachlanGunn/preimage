use crate::objects::git::*;
use tokio::prelude::*;

pub struct RepoWalker<'walk> {
    repo: git2::Repository,
    iter: *mut git2::Revwalk<'walk>,
}

unsafe impl<'walk> Send for RepoWalker<'walk> {}

impl<'walk> Drop for RepoWalker<'walk> {
    fn drop(&mut self) {
        drop(unsafe { Box::from_raw(self.iter) });
    }
}

impl<'walk> RepoWalker<'walk> {
    pub fn new(path: ::std::path::PathBuf) -> Result<Self, std::io::Error> {
        let owned_path = path.to_path_buf();
        let repo = match git2::Repository::open(owned_path) {
            Ok(r) => r,
            Err(_) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to open repository",
                ));
            }
        };

        let iter = {
            let mut iter = Box::from(repo.revwalk().or_else(|_| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to start repository walk",
                ))
            })?);

            iter.push_head().or_else(|_| {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to start walk at head",
                ))
            })?;

            Box::leak(iter) as *mut git2::Revwalk
        };

        Ok(RepoWalker { repo, iter })
    }
}

impl<'walk> Stream for RepoWalker<'walk> {
    type Item = Box<crate::objects::Object>;
    type Error = ::std::io::Error;

    fn poll(&mut self) -> Poll<Option<Box<crate::objects::Object>>, ::std::io::Error> {
        let iter = unsafe { &mut *self.iter };

        let rev = match iter.next() {
            Some(rev) => rev,
            None => return Ok(Async::Ready(None)),
        };

        let commit_id = match rev {
            Ok(x) => x,
            Err(_) => return Ok(Async::Ready(None)),
        };

        let commit_info = GitCommit {
            path: self.repo.path().to_path_buf(),
            id: commit_id,
        };

        Ok(Async::Ready(Some(Box::from(commit_info))))
    }
}
