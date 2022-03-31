//! Data structures for easier interactions with the filesystem.
use byte_unit::Byte;
use serde::{Deserialize, Serialize};
use std::{
    fs::FileType,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ItemType {
    File,
    Dir,
    Symlink,
}

impl Default for ItemType {
    fn default() -> Self {
        ItemType::File
    }
}

impl From<FileType> for ItemType {
    fn from(tp: FileType) -> Self {
        if tp.is_dir() {
            ItemType::Dir
        } else if tp.is_file() {
            ItemType::File
        } else if tp.is_symlink() {
            ItemType::Symlink
        } else {
            unreachable!()
        }
    }
}

/// An Item represents a single element of the filesystem.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Item {
    path: PathBuf,
    tp: ItemType,
    size: Option<Byte>,
}

impl Item {
    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn name(&self) -> Option<String> {
        self.path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
    }
    pub fn ext(&self) -> Option<String> {
        self.path
            .extension()
            .map(|s| s.to_string_lossy().into_owned())
    }
    pub fn tp(&self) -> &ItemType {
        &self.tp
    }
    pub fn size(&mut self) -> anyhow::Result<Byte> {
        // If file size is cached, return it
        if let Some(size) = self.size {
            Ok(size)
        // Otherwise, calculate it
        } else {
            let bytes = Byte::from_bytes(fs_extra::dir::get_size(&self.path)?.into());
            self.size = Some(bytes);
            Ok(bytes)
        }
    }
}
