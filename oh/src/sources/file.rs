use tokio::prelude::*;
use walkdir::WalkDir;
// &Fn(walkdir::DirEntry) -> bool
struct Walker<'walk> {
    iter: walkdir::IntoIter,
    app: crate::config::OHApp,
    git_walker: Option<crate::sources::git::RepoWalker<'walk>>,
}

fn is_hidden(entry: &::walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map_or(false, |s| s != ".git" && s.starts_with('.'))
}

impl<'walk> Walker<'walk> {
    fn new(
        dir: std::path::PathBuf,
        app: crate::config::OHApp,
    ) -> Result<Self, Box<::std::error::Error>> {
        let root = dir.canonicalize()?;
        Ok(Walker {
            iter: WalkDir::new(root).into_iter(),
            app,
            git_walker: None,
        })
    }
}

pub fn walk_dir<S: crate::sinks::ObjectSink + Send + Sync + 'static>(
    dir: &::std::path::Path,
    sink: S,
    app: &crate::config::OHApp,
) {
    let stream = match Walker::new(dir.to_path_buf(), app.clone()) {
        Ok(w) => w,
        Err(_) => {
            return;
        }
    };

    tokio::run(
        stream
            .for_each(move |path| {
                sink.push(&*path);
                Ok(())
            })
            .or_else(|_| Err(())),
    );
}

impl<'walk> Stream for Walker<'walk> {
    type Item = Box<crate::objects::Object>;
    type Error = ::std::io::Error;

    fn poll(&mut self) -> Poll<Option<Box<crate::objects::Object>>, ::std::io::Error> {
        if let Some(ref mut git_walker) = self.git_walker {
            let result = git_walker.poll();
            if let Ok(futures::Async::Ready(None)) = result {
                self.git_walker = None;
            } else {
                return result;
            }
        }

        loop {
            let entry = match self.iter.next() {
                None => {
                    return Ok(futures::Async::Ready(None));
                }
                Some(Ok(entry)) => entry,
                Some(Err(_)) => {
                    continue;
                }
            };

            if self.app.exclude_paths.contains(entry.path()) {
                self.iter.skip_current_dir();
            } else if entry.file_type().is_dir()
                && entry
                    .path()
                    .file_name()
                    .unwrap_or(&::std::ffi::OsStr::new(""))
                    == ".git"
            {
                if entry.file_type().is_dir() {
                    self.iter.skip_current_dir();
                }

                let parent_dir = entry
                    .path()
                    .parent()
                    .expect("Failed to get parent path")
                    .to_path_buf();
                self.git_walker = Some(crate::sources::git::RepoWalker::new(parent_dir)?);
            } else if is_hidden(&entry) {
                if entry.file_type().is_dir() {
                    self.iter.skip_current_dir();
                }
            } else {
                return Ok(futures::Async::Ready(Some(Box::from(
                    crate::objects::file::SimpleFile {
                        path: ::std::path::PathBuf::from(entry.path()),
                    },
                ))));
            }
        }
    }
}
