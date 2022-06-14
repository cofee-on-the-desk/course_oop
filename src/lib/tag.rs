//! Tags represent a category of files that meet a certain criteria.
use std::path::Path;

use crate::lib::Item;

use serde::{Deserialize, Serialize};

use super::ItemType;

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
    pub fn is(&self, path: &Path) -> anyhow::Result<bool> {
        let item = Item::try_from_path(path)?;
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
                    result = result && basis.is(item.path())?
                }
                Ok(result)
            }
            Basis::Or(vec) => {
                let mut result = true;
                for basis in vec {
                    result = result || basis.is(item.path())?
                }
                Ok(result)
            }
        }
    }
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
    fn is(&self, path: &Path) -> anyhow::Result<bool> {
        Ok(self.tag.is(path)? == self.used)
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
    pub fn is(&self, path: &Path) -> anyhow::Result<bool> {
        let mut result = self.0.is(path)?;
        for single in &self.1 {
            result = result && single.is(path)?;
        }
        Ok(result)
    }
    pub fn name(&self) -> String {
        std::iter::once(&self.0)
            .chain(self.1.iter())
            .map(|single| single.name())
            .collect::<Vec<_>>()
            .join(" + ")
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
    name: String,
    desc: String,
    basis: Basis,
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
    pub fn is(&self, path: &Path) -> anyhow::Result<bool> {
        self.basis.is(path)
    }
    pub fn dummy() -> Self {
        Tag { name: "🧱 Dummy".into(), basis: Basis::Name("dummy.test".into()), desc: "An object with the name 'dummy.test'. Used as a placeholder inside events, usually you would want to replace it with another useful tag.".into() }
    }
}

pub fn all_tags() -> Vec<Tag> {
    vec![
    Tag::dummy(),
    Tag { name: "📁 Folder".into(), basis: Basis::Type(ItemType::Dir), desc: "An object that contains other files.".into() },
    Tag { name: "📄 File".into(), basis: Basis::Type(ItemType::File), desc: "An object that contains data. The data can be represented in plain text or encoded in any format.".into() },
    Tag { name: "🐚 Empty".into(), basis: Basis::ChildrenCount(0), desc: "An empty folder.".into() },
    Tag { name: "📦 Item".into(), basis: Basis::Bool(true), desc: "A folder, file or a symlink.".into() }
    ]
}
