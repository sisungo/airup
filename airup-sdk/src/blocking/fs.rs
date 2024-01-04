//! Filesystem utilities.

use crate::util::IterExt;
use std::{
    borrow::Cow,
    collections::HashSet,
    ffi::OsString,
    path::{Path, PathBuf},
};

/// Represents to a "directory chain", which has a filesystem layout similar to:
/// ```text
/// /dir_chain
///           /file1.txt
///           /file2.txt
///           /chain_next -> /dir_chain1
/// /dir_chain1
///            /file1.txt
///            /file3.txt
///            /file4.txt
/// ...
/// ```
/// When finding a file or directory from the chain, the program will iterate over each directory in the chain, until the
/// matching file or directory is found. For example, in the chain above, finding `file1.txt` returns `/dir_chain/file1.txt`,
/// and finding `file3.txt` returns `/dir_chain1/file3.txt`.
#[derive(Debug, Clone)]
pub struct DirChain<'a>(Cow<'a, Path>);
impl<'a> DirChain<'a> {
    pub fn new<P: Into<Cow<'a, Path>>>(path: P) -> Self {
        Self(path.into())
    }

    /// Find a file by filename.
    pub fn find<P: AsRef<Path>>(&self, path: P) -> Option<PathBuf> {
        let mut pwd = self.0.clone();
        let path = path.as_ref();

        loop {
            let path = pwd.join(path);
            if path.exists() {
                return Some(path);
            } else {
                let path = pwd.join("chain_next");
                if path.exists() {
                    pwd = path.into();
                } else {
                    return None;
                }
            }
        }
    }

    /// Returns path of end of the chain.
    pub fn end(&self) -> PathBuf {
        let mut pwd = self.0.clone();

        loop {
            let chain_next = pwd.join("chain_next");
            if chain_next.exists() {
                pwd = chain_next.into();
            } else {
                break pwd.into();
            }
        }
    }

    /// Gets a list that contains relative paths of filesystem objects on the chain. The result is sorted (chain-order first).
    ///
    /// # Errors
    /// An `Err(_)` is returned if the underlying filesystem operation failed.
    pub fn read_chain(&self) -> std::io::Result<Vec<OsString>> {
        let mut result = Vec::new();
        let mut pwd = self.0.clone();
        let mut elements = HashSet::new();
        let mut unsorted = Vec::new();

        loop {
            let mut should_continue = false;

            let mut read_dir = std::fs::read_dir(&pwd)?;
            while let Some(Ok(entry)) = read_dir.next() {
                let file_name = entry.file_name();
                if file_name == "chain_next" {
                    should_continue = true;
                } else {
                    elements.insert(file_name);
                }
            }

            elements.drain().for_each(|x| unsorted.push(x));
            unsorted.sort_unstable();
            result.append(&mut unsorted);

            if should_continue {
                pwd = pwd.join("chain_next").into();
            } else {
                break;
            }
        }

        Ok(result.into_iter().dedup_all())
    }

    /// Finds a file from the chain, or creates it at the end of the chain if not found.
    ///
    /// # Errors
    /// An `Err(_)` is returned if the underlying filesystem operation failed.
    pub fn find_or_create<P: AsRef<Path>>(&self, path: P) -> std::io::Result<PathBuf> {
        let path = path.as_ref();
        if let Some(np) = self.find(path) {
            Ok(np)
        } else {
            let np = self.end().join(path);
            std::fs::File::create(&np)?;
            Ok(np)
        }
    }
}
impl From<PathBuf> for DirChain<'static> {
    fn from(value: PathBuf) -> Self {
        Self::new(value)
    }
}
