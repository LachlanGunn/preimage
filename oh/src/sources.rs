use futures::prelude::*;
use walkdir::WalkDir;
// &Fn(walkdir::DirEntry) -> bool
struct Walker<'walk> {
    iter: walkdir::IntoIter,
    app: &'walk crate::config::OHApp,
}

fn is_hidden(entry: &::walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map_or(false, |s| s != ".git" && s.starts_with('.'))
}

impl<'walk> Walker<'walk> {
    fn new(
        dir: &::std::path::Path,
        app: &'walk crate::config::OHApp,
    ) -> Result<Self, Box<::std::error::Error>> {
        let root = dir.canonicalize()?;
        Ok(Walker {
            iter: WalkDir::new(root).into_iter(),
            app,
        })
    }
}

pub fn walk_dir(
    dir: &::std::path::Path,
    sink: &mut crate::sinks::ObjectSink,
    app: &crate::config::OHApp,
) {
    let mut stream = match Walker::new(dir, app) {
        Ok(w) => w,
        Err(_) => {
            return;
        }
    };

    while let Ok(Async::Ready(Some(path))) = stream.poll() {
        sink.push(&*path);
    }
}

impl<'walk> Stream for Walker<'walk> {
    type Item = Box<crate::objects::Object>;
    type Error = ::std::io::Error;

    fn poll(&mut self) -> Poll<Option<Box<crate::objects::Object>>, ::std::io::Error> {
        loop {
            let entry = match self.iter.next() {
                None => {
                    return Err(::std::io::Error::new(
                        ::std::io::ErrorKind::Other,
                        "No objects available.",
                    ));
                }
                Some(Ok(entry)) => entry,
                Some(Err(_)) => {
                    continue;
                }
            };

            if self.app.exclude_paths.contains(entry.path()) {
                self.iter.skip_current_dir();
            } else if (entry.file_type().is_dir()
                && entry
                    .path()
                    .file_name()
                    .unwrap_or(&::std::ffi::OsStr::new(""))
                    == ".git")
                || is_hidden(&entry)
            {
                if entry.file_type().is_dir() {
                    self.iter.skip_current_dir();
                }
            //::objects::git::walk_repo(entry.path().parent().unwrap(), sink);
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
