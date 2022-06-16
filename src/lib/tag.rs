//! Tags represent a category of files that meet a certain criteria.
use std::{cmp::Ordering, path::Path, time::Duration};

use crate::{lib::Item, util::PathExt};

use anyhow::Context;
use byte_unit::Byte;
use infer::MatcherType;
use serde::{Deserialize, Serialize};

use super::FileType;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Base {
    Type(FileType),
    Name(String),
    SizeLT(Byte),
    SizeGT(Byte),
    Extension(Vec<String>),
    // We have to add a separate variant for each ordering
    // because `std::cmp::Ordering` does not implement serde traits.
    ChildrenCountLT(usize),
    ChildrenCountET(usize),
    ChildrenCountGT(usize),
    LifetimeLT(Duration),
    LifetimeGT(Duration),
    IsImage,
    IsVideo,
    IsAudio,
    IsDocument,
    IsArchive,
    IsBook,
}

impl Base {
    pub fn is(&self, item: &mut Item) -> anyhow::Result<bool> {
        match self {
            Base::Type(file_type) => Ok(item.file_type() == file_type),
            Base::Name(name) => Ok(item.name().as_ref() == Some(name)),
            Base::Extension(extensions) => Ok(item.file_type() == &FileType::File
                && item
                    .path()
                    .ext()
                    .map(|ext| extensions.contains(&ext))
                    .unwrap_or(false)),
            Base::SizeLT(byte) => is_size(item, Ordering::Less, byte),
            Base::SizeGT(byte) => is_size(item, Ordering::Greater, byte),
            Base::ChildrenCountLT(count) => is_children_count(item.path(), Ordering::Less, count),
            Base::ChildrenCountET(count) => is_children_count(item.path(), Ordering::Equal, count),
            Base::ChildrenCountGT(count) => {
                is_children_count(item.path(), Ordering::Greater, count)
            }
            Base::LifetimeLT(duration) => is_lifetime(item.path(), Ordering::Less, duration),
            Base::LifetimeGT(duration) => is_lifetime(item.path(), Ordering::Greater, duration),
            Base::IsImage => is_matcher_type(item.path(), MatcherType::Image),
            Base::IsVideo => is_matcher_type(item.path(), MatcherType::Video),
            Base::IsAudio => is_matcher_type(item.path(), MatcherType::Audio),
            Base::IsDocument => is_matcher_type(item.path(), MatcherType::Doc),
            Base::IsArchive => is_matcher_type(item.path(), MatcherType::Archive),
            Base::IsBook => is_matcher_type(item.path(), MatcherType::Book),
        }
    }
}

fn is_lifetime(path: &Path, ordering: Ordering, duration: &Duration) -> anyhow::Result<bool> {
    let then = std::fs::metadata(path)?.created()?;
    let now = std::time::SystemTime::now();
    let dur = now.duration_since(then)?;
    Ok(dur.cmp(duration) == ordering)
}

fn is_size(item: &mut Item, ordering: Ordering, size: &Byte) -> anyhow::Result<bool> {
    Ok(item.size()?.cmp(size) == ordering)
}

fn is_children_count(path: &Path, ordering: Ordering, count: &usize) -> anyhow::Result<bool> {
    Ok(path.is_dir() && std::fs::read_dir(path)?.count().cmp(count) == ordering)
}

fn is_matcher_type(path: &Path, tp: MatcherType) -> anyhow::Result<bool> {
    Ok(infer::get_from_path(path)?
        .with_context(|| "Unknown file format")?
        .matcher_type()
        == tp)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct SingleTag {
    tag: Tag,
    used: bool,
}

impl Default for SingleTag {
    fn default() -> Self {
        SingleTag {
            tag: Tag::default(),
            used: true,
        }
    }
}

impl SingleTag {
    fn is(&self, item: &mut Item) -> anyhow::Result<bool> {
        Ok(self.tag.is(item)? == self.used)
    }
    fn name(&self) -> String {
        if self.used {
            self.tag.name().to_owned()
        } else {
            format!("NOT({})", self.tag.name())
        }
    }
    fn desc(&self) -> String {
        self.tag.desc().to_owned()
    }
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct TagExpr(SingleTag, Vec<SingleTag>);

impl TagExpr {
    pub fn new(tag: Tag, used: bool) -> Self {
        TagExpr(SingleTag { tag, used }, Vec::new())
    }
    pub fn is(&self, item: &mut Item) -> anyhow::Result<bool> {
        let mut result = self.0.is(item)?;
        for single in &self.1 {
            result = result && single.is(item)?;
        }
        Ok(result)
    }
    pub fn name(&self) -> String {
        std::iter::once(&self.0)
            .chain(self.1.iter())
            .map(|single| single.name())
            .collect::<Vec<_>>()
            .join(" AND ")
    }
    pub fn desc(&self) -> String {
        if self.1.is_empty() {
            self.0.desc()
        } else {
            std::iter::once(&self.0)
                .chain(self.1.iter())
                .map(|single| single.desc())
                .collect::<Vec<_>>()
                .join("\n")
        }
    }
    pub fn has(&self, t: &Tag) -> bool {
        &self.0.tag == t || self.1.iter().any(|single| &single.tag == t)
    }
    pub fn remove(&mut self, t: &Tag) {
        if &self.0.tag == t && !self.1.is_empty() {
            self.0 = self.1.remove(0);
        } else if let Some(index) = self.1.iter().position(|single| &single.tag == t) {
            self.1.remove(index);
        }
    }
    pub fn push(&mut self, tag: Tag, used: bool) {
        self.1.push(SingleTag { tag, used })
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Tag {
    pub name: String,
    pub desc: String,
    pub basis: Base,
}

impl Default for Tag {
    fn default() -> Self {
        Tag::dummy()
    }
}

impl Tag {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn desc(&self) -> &str {
        &self.desc
    }
    pub fn is(&self, item: &mut Item) -> anyhow::Result<bool> {
        self.basis.is(item)
    }
    pub fn dummy() -> Self {
        Tag { name: "🧱 Dummy".into(), basis: Base::Name("dummy.test".into()), desc: "An object with the name 'dummy.test'. Used as a placeholder inside events, usually you would want to replace it with another useful tag.".into() }
    }
}

pub fn all_tags() -> Vec<Tag> {
    all_tags_sorted_by_columns().into_iter().flatten().collect()
}

pub fn all_tags_sorted_by_columns() -> [Vec<Tag>; 4] {
    [
        vec![
            Tag { name: "📁 Folder".into(), basis: Base::Type(FileType::Dir), desc: "An object that contains other files.".into() },
            Tag { name: "📄 File".into(), basis: Base::Type(FileType::File), desc: "An object that contains data. The data can be represented in plain text or encoded in any format.".into() },
            Tag { name: "🖼️ Image".into(), basis: Base::IsImage, desc: "A file that contains graphics.".into() },
            Tag { name: "🎞️ Video".into(), basis: Base::IsVideo, desc: "A file that contains video materials.".into() },
            Tag { name: "🔉 Audio".into(), basis: Base::IsAudio, desc: "A file that contains audio.".into() },
            Tag { name: "🗃️ Archive".into(), basis: Base::IsArchive, desc: "A compressed file format.".into() },
            Tag { name: "📃 Document".into(), basis: Base::IsDocument, desc: "A file recognizable by an office suite, such as a text document, presentation or a spreadsheet.".into() },
            Tag { name: "📚 Book".into(), basis: Base::IsBook, desc: "A document that is recognizable by book readers.".into() },
        ],
        vec![
            Tag { name: "💾 < 1KB".into(), basis: Base::SizeLT(Byte::from_str("1KB").unwrap()), desc: "Various files that have their total size less than 1KB. Size for folders is calculated recursively.".into()    },
            Tag { name: "💾 < 1MB".into(), basis: Base::SizeLT(Byte::from_str("1MB").unwrap()), desc: "Various files that have their total size less than 1MB. Size for folders is calculated recursively.".into()    },
            Tag { name: "💾 < 1GB".into(), basis: Base::SizeLT(Byte::from_str("1GB").unwrap()), desc: "Various files that have their total size less than 1GB. Size for folders is calculated recursively.".into()    },
            Tag { name: "💾 < 10GB".into(), basis: Base::SizeLT(Byte::from_str("10GB").unwrap()), desc: "Various files that have their total size less than 10GB. Size for folders is calculated recursively.".into() },
        ],
        vec![
            Tag { name: "🕒 Lifetime > 1h".into(), basis: Base::LifetimeGT(Duration::from_secs(360)), desc: "Files that were created more than 1 hour ago".into() },
            Tag { name: "🕒 Lifetime > 8h".into(), basis: Base::LifetimeGT(Duration::from_secs(28800)), desc: "Files that were created more than 8 hours ago".into() },
            Tag { name: "🕒 Lifetime > 24h".into(), basis: Base::LifetimeGT(Duration::from_secs(86400)), desc: "Files that were created more than 24 hours ago".into() }, 
        ],
        vec![
            Tag { name: "📂 Empty Folder".into(),  basis: Base::ChildrenCountET(0), desc: "An empty folder.".into() },
            Tag::dummy(),
        ]
    ]
}
