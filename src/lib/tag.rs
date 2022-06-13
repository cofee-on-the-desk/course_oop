//! Tags represent a category of files that meet a certain criteria.
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

/*
enum Tag {
    File,
    Directory,
    DirectoryEmpty,
    DirectoryFilesLessThen,
    Symlink,
    LessThen1MB,
    LessThen100MB,
    LessThen1GB,
    LessThen10GB,
    Image,
    Video,
    Audio,
    Document,
    Book,
    TextDocument,
    Presentation,
    Table,
    Office,
    Archive,
    And(Box<Tag>, Box<Tag>),
    Not(Box<Tag>),
}
*/

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Tag {
    name: String,
    desc: String,
    basis: Basis,
}

impl Default for Tag {
    fn default() -> Self {
        Tag { name: "🧱 Dummy".into(), basis: Basis::Name("dummy.test".into()),
    desc: "An object with the name 'dummy.text'. Used as the placeholder inside event, usually you would want to replace it with another useful tag.".into()
}
    }
}

impl Tag {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn desc(&self) -> &str {
        &self.desc
    }
    pub fn is(&self, entry: &Item) -> anyhow::Result<bool> {
        self.basis.is(entry)
    }
}

pub fn all_tags() -> Vec<Tag> {
    vec![
    Tag { name: "🧱 Dummy".into(), basis: Basis::Name("dummy.test".into()),
    desc: "An object with the name 'dummy.test'. Used as the placeholder inside event, usually you would want to replace it with another useful tag.".into()},
    Tag { name: "📁 Folder".into(), basis: Basis::Type(ItemType::Dir), desc: "An object that contains other files.".into(), },
            Tag { name: "📄 File".into(), basis: Basis::Type(ItemType::File)
            , desc: "An object that contains data. The data can be represented in plain text or encoded in any format.".into(),
            },
            Tag { name: "🐚 Empty".into(), basis: Basis::ChildrenCount(0)
            , desc: "An empty folder.".into()
               },
               Tag { name: "📦 Item".into(), basis: Basis::Bool(true),
               desc: "A folder, file or a symlink.".into()
               }
    ]
}
