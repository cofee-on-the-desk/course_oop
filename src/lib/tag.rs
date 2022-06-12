//! Tags represent a category of files that meet a certain criteria.
use crate::lib::Item;

use serde::{Deserialize, Serialize};

use super::ItemType;

pub struct TagBuilder(Tag);

impl TagBuilder {
    pub fn new(name: &'static str, basis: Basis) -> Self {
        TagBuilder(Tag {
            name: name.to_string(),
            emoji: None,
            desc: None,
            basis,
        })
    }
    pub fn emoji(mut self, emoji: &'static str) -> Self {
        self.0.emoji = Some(emoji.to_string());
        self
    }
    pub fn desc(mut self, desc: &'static str) -> Self {
        self.0.desc = Some(desc.to_string());
        self
    }
    pub fn build(self) -> Tag {
        self.0
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Basis {
    Bool(bool),
    Type(ItemType),
    Name(String),
    Extension(Vec<String>),
    ChildrenCount(usize),
    And(Vec<Basis>),
    Or(Vec<Basis>),
}

impl Basis {
    pub fn is(&self, item: &Item) -> anyhow::Result<bool> {
        match self {
            Basis::Bool(b) => Ok(*b),
            Basis::Type(tp) => Ok(item.tp() == tp),
            Basis::Name(name) => Ok(item.name().as_ref() == Some(name)),
            Basis::Extension(extensions) => Ok(item.tp() == &ItemType::File
                && item
                    .ext()
                    .map(|ext| extensions.contains(&ext))
                    .unwrap_or(false)),
            Basis::ChildrenCount(count) => Ok(
                item.tp() == &ItemType::Dir && std::fs::read_dir(item.path())?.count() == *count
            ),
            Basis::And(vec) => {
                let mut result = true;
                for basis in vec {
                    result = result && basis.is(item)?
                }
                Ok(result)
            }
            Basis::Or(vec) => {
                let mut result = true;
                for basis in vec {
                    result = result || basis.is(item)?
                }
                Ok(result)
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Tag {
    name: String,
    emoji: Option<String>,
    desc: Option<String>,
    basis: Basis,
}

impl Default for Tag {
    fn default() -> Self {
        common::empty()
    }
}

impl Tag {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn emoji_name(&self) -> String {
        if let Some(emoji) = &self.emoji {
            format!("{} {}", emoji, self.name)
        } else {
            self.name.clone()
        }
    }
    pub fn desc(&self) -> Option<&str> {
        self.desc.as_deref()
    }
    pub fn is(&self, entry: &Item) -> anyhow::Result<bool> {
        self.basis.is(entry)
    }
}

pub mod common {
    use super::{Basis, Tag, TagBuilder};
    use crate::lib::ItemType;

    pub fn folder() -> Tag {
        TagBuilder::new("Folder", Basis::Type(ItemType::Dir))
            .emoji("📁")
            .desc("An object that contains other files.")
            .build()
    }
    pub fn file() -> Tag {
        TagBuilder::new("File", Basis::Type(ItemType::File))
            .emoji("📄")
            .desc(
                "An object that contains data. The data can be represented in plain text or encoded in any format.",
            )
            .build()
    }
    pub fn link() -> Tag {
        TagBuilder::new("Symlink", Basis::Type(ItemType::Symlink))
            .emoji("🔗")
            .desc("An object that points to another object.")
            .build()
    }
    pub fn empty() -> Tag {
        TagBuilder::new("Empty", Basis::ChildrenCount(0))
            .emoji("🐚")
            .desc("An empty folder.")
            .build()
    }
    pub fn item() -> Tag {
        TagBuilder::new("Item", Basis::Bool(true))
            .emoji("📦")
            .desc("A folder, file or a symlink.")
            .build()
    }
    pub fn never() -> Tag {
        TagBuilder::new("Never", Basis::Bool(false))
            .emoji("🌑")
            .desc(
                "A never tag means an empty set, so there is no such file \nthat can fit this tag. Can be used for testing purposes.",
            )
            .build()
    }

    pub fn all() -> Vec<Tag> {
        vec![folder(), file(), link(), empty(), item(), never()]
    }
}
