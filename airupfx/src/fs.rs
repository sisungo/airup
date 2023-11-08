//! Filesystem utilities.

use crate::util::IterExt;
use std::{
    borrow::Cow,
    collections::HashSet,
    ffi::OsString,
    path::{Path, PathBuf},
};
use tokio::io::AsyncReadExt;

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
    pub async fn find<P: AsRef<Path>>(&self, path: P) -> Option<PathBuf> {
        let mut pwd = self.0.clone();
        let path = path.as_ref();

        loop {
            let path = pwd.join(path);
            if tokio::fs::try_exists(&path).await.unwrap_or_default() {
                return Some(path);
            } else {
                let path = pwd.join("chain_next");
                if tokio::fs::try_exists(&path).await.unwrap_or_default() {
                    pwd = path.into();
                } else {
                    return None;
                }
            }
        }
    }

    /// Returns path of end of the chain.
    pub async fn end(&self) -> PathBuf {
        let mut pwd = self.0.clone();

        loop {
            let chain_next = pwd.join("chain_next");
            if tokio::fs::try_exists(&chain_next).await.unwrap_or_default() {
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
    pub async fn read_chain(&self) -> std::io::Result<Vec<OsString>> {
        let mut result = Vec::new();
        let mut pwd = self.0.clone();
        let mut elements = HashSet::new();
        let mut unsorted = Vec::new();

        loop {
            let mut should_continue = false;

            let mut read_dir = tokio::fs::read_dir(&pwd).await?;
            while let Ok(Some(entry)) = read_dir.next_entry().await {
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
    pub async fn find_or_create<P: AsRef<Path>>(&self, path: P) -> std::io::Result<PathBuf> {
        let path = path.as_ref();
        if let Some(np) = self.find(path).await {
            Ok(np)
        } else {
            let np = self.end().await.join(path);
            tokio::fs::File::create(&np).await?;
            Ok(np)
        }
    }
}
impl From<PathBuf> for DirChain<'static> {
    fn from(value: PathBuf) -> Self {
        Self::new(value)
    }
}

/// Like [`tokio::fs::read_to_string`], but max size of file is limited.
pub async fn read_to_string_limited(
    path: impl AsRef<Path>,
    limit: usize,
) -> std::io::Result<String> {
    let file = tokio::fs::File::open(path).await?;
    let mut capacity = file.metadata().await?.len() as usize;
    if capacity > limit {
        capacity = limit;
    }
    let mut buffer = String::with_capacity(capacity);
    file.take(limit as _).read_to_string(&mut buffer).await?;
    Ok(buffer)
}

/// Commits filesystem caches to disk.
pub async fn sync() {
    crate::sys::fs::sync().await
}

/// Sets a file with socket permissions.
pub async fn set_sock_permission<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    crate::sys::fs::set_sock_permission(path).await
}
