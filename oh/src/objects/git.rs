extern crate data_encoding;
extern crate git2;
extern crate sha2;

use objects::Object;

use sha2::Digest;

pub struct GitCommit {
    pub path: ::std::path::PathBuf,
    pub id: git2::Oid,
}

impl ::objects::Object for GitCommit {
    fn hash(&self) -> Result<::std::vec::Vec<::std::vec::Vec<u8>>, Box<::std::error::Error>> {
        Ok(vec![::std::vec::Vec::from(self.id.as_bytes())])
    }

    fn to_str(&self) -> String {
        format!("git:{}", self.path.to_str().unwrap())
    }
}

pub struct GitFile<'a> {
    pub commit: &'a GitCommit,
    pub path: &'a str,
    pub hash: &'a [u8],
    pub id: &'a [u8],
}

impl<'a> ::objects::Object for GitFile<'a> {
    fn hash(&self) -> Result<::std::vec::Vec<::std::vec::Vec<u8>>, Box<::std::error::Error>> {
        Ok(vec![
            ::std::vec::Vec::from(self.id),
            ::std::vec::Vec::from(self.hash),
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

pub fn walk_tree(
    prefix: &str,
    tree: &git2::Tree,
    commit: &git2::Commit,
    commit_info: &GitCommit,
    repository: &git2::Repository,
    sink: &mut ::sinks::ObjectSink,
) {
    for entry in tree.iter() {
        let name = match entry.name() {
            Some(name) => name,
            None => continue,
        };

        let object = match entry.to_object(repository) {
            Ok(object) => object,
            Err(_) => continue,
        };

        let mut full_path = prefix.to_owned();
        full_path.push_str("/");
        full_path.push_str(name);

        match object.kind() {
            Some(git2::ObjectType::Tree) => {
                let new_tree = match object.peel_to_tree() {
                    Ok(t) => t,
                    Err(_) => continue,
                };

                walk_tree(&full_path, &new_tree, commit, commit_info, repository, sink);
                continue;
            }
            Some(git2::ObjectType::Blob) => {}
            _ => continue,
        }

        let blob = match object.peel_to_blob() {
            Ok(blob) => blob,
            Err(_) => continue,
        };

        if sink.lookup(blob.id().as_bytes()).is_ok() {
            continue;
        }

        let mut hasher = sha2::Sha256::default();
        hasher.input(blob.content());

        sink.push(&GitFile {
            commit: commit_info,
            path: &full_path,
            hash: &hasher.result()[..],
            id: &blob.id().as_bytes(),
        });
    }
}

pub fn walk_repo(path: &::std::path::Path, sink: &mut ::sinks::ObjectSink) {
    let repo = match git2::Repository::open(path) {
        Ok(x) => x,
        Err(_) => return,
    };

    let mut revwalk = match repo.revwalk() {
        Ok(r) => r,
        Err(_) => return,
    };

    if revwalk.push_head().is_err() {
        return;
    }

    for rev in revwalk {
        let commit_id = match rev {
            Ok(x) => x,
            Err(_) => break,
        };

        let commit = match repo.find_commit(commit_id) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let commit_info = GitCommit {
            path: ::std::path::PathBuf::from(path),
            id: commit_id,
        };

        let commit_tree = match commit.tree() {
            Ok(x) => x,
            Err(_) => continue,
        };

        if let Ok(result) = sink.lookup(commit_id.as_bytes()) {
            if *result == *commit_info.to_str() {
                continue;
            }
        }
        walk_tree("", &commit_tree, &commit, &commit_info, &repo, sink);
        sink.push(&commit_info);
    }
}
