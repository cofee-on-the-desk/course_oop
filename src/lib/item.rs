//! Data structures for easier interactions with the filesystem.
use byte_unit::Byte;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::util::PathExt;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum FileType {
    File,
    Dir,
    Symlink,
}

impl Default for FileType {
    fn default() -> Self {
        FileType::File
    }
}

impl From<fs::FileType> for FileType {
    fn from(tp: fs::FileType) -> Self {
        if tp.is_dir() {
            FileType::Dir
        } else if tp.is_file() {
            FileType::File
        } else if tp.is_symlink() {
            FileType::Symlink
        } else {
            unreachable!()
        }
    }
}

/// Snapshot of information about a certain file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    /// Path to the file.
    path: PathBuf,
    /// Type of the file. Only files,
    /// directories and symlinks are supported.
    file_type: FileType,
    /// File size is usually not calculated,
    /// as it might be too expensive to do so.
    ///
    /// If you want to calculate it,
    /// the whole snapshot of the file will
    /// be updated.
    size: Option<Byte>,
    // Time when the file was created.
    creation_time: SystemTime,
    // Time when the file was modified.
    modified_time: SystemTime,
}

impl Item {
    pub fn new(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let metadata = std::fs::symlink_metadata(path)?;
        let file_type = FileType::from(metadata.file_type());
        Ok(Item {
            path: path.to_owned(),
            file_type,
            size: None,
            creation_time: metadata.created()?,
            modified_time: metadata.modified()?,
        })
    }
    pub fn new_with_size(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let size = Some(Byte::from_bytes(fs_extra::dir::get_size(&path)?.into()));
        Ok(Item {
            size,
            ..Item::new(path)?
        })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn name(&self) -> Option<String> {
        self.path.name()
    }
    pub fn ext(&self) -> Option<String> {
        self.path.ext()
    }
    pub fn file_type(&self) -> &FileType {
        &self.file_type
    }
    pub fn size(&mut self) -> anyhow::Result<Byte> {
        // If the size is cached, return it
        if let Some(size) = self.size {
            Ok(size)
        // Otherwise, update the snapshot
        } else {
            *self = Item::new_with_size(self.path())?;
            Ok(self.size.unwrap())
        }
    }
}
