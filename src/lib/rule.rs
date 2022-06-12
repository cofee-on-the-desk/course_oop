//! Data structures and utilities related to the rule system.
use crate::lib::tag::{common, Tag};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rule {
    title: String,
    condition: Tag,
    events: Vec<Event>,
}

impl Default for Rule {
    fn default() -> Self {
        Rule {
            title: "New Rule".into(),
            condition: Tag::default(),
            events: Vec::new(),
        }
    }
}

impl Rule {
    pub fn new(condition: Tag, events: Vec<Event>) -> Self {
        Rule {
            title: "New Rule".into(),
            condition,
            events,
        }
    }
    pub fn condition(&self) -> &Tag {
        &self.condition
    }
    pub fn events(&self) -> &[Event] {
        &self.events[..]
    }
    pub fn condition_mut(&mut self) -> &mut Tag {
        &mut self.condition
    }
    pub fn events_mut(&mut self) -> &mut Vec<Event> {
        &mut self.events
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn title_mut(&mut self) -> &mut String {
        &mut self.title
    }
}

use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CopyOptions {
    pub overwrite: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoveOptions {
    pub overwrite: bool,
}

impl Default for CopyOptions {
    fn default() -> Self {
        CopyOptions { overwrite: false }
    }
}

impl Default for MoveOptions {
    fn default() -> Self {
        MoveOptions { overwrite: false }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Copy {
        tag: Tag,
        target: PathBuf,
        options: CopyOptions,
    },
    Move {
        tag: Tag,
        target: PathBuf,
        options: MoveOptions,
    },
    Idle,
}

impl Default for Event {
    fn default() -> Self {
        Event::Idle
    }
}

impl Event {
    pub fn name(&self) -> &str {
        match &self {
            Event::Copy { .. } => "Copy",
            Event::Move { .. } => "Move",
            Event::Idle => "Idle",
        }
    }
    pub fn gtk_icon(&self) -> &str {
        match &self {
            Event::Copy { .. } => "edit-copy-symbolic",
            Event::Move { .. } => "go-jump-symbolic",
            Event::Idle => "preferences-desktop-screensaver-symbolic",
        }
    }
    pub fn vars(&self) -> Vec<Var> {
        match &self {
            Event::Copy {
                tag,
                target,
                options,
            } => {
                let mut vars = vec![
                    Var::String {
                        label: "Copy".into(),
                        css_class: Some("bold"),
                    },
                    Var::Tag(tag.clone()),
                    Var::String {
                        label: "to".into(),
                        css_class: Some("opaque"),
                    },
                    Var::Path(target.into()),
                ];
                if options.overwrite {
                    vars.push(Var::String {
                        label: "(overwrite)".into(),
                        css_class: Some("opaque"),
                    });
                }
                vars
            }
            Event::Move {
                tag,
                target,
                options,
            } => {
                let mut vars = vec![
                    Var::String {
                        label: "Move".into(),
                        css_class: Some("bold"),
                    },
                    Var::Tag(tag.clone()),
                    Var::String {
                        label: "to".into(),
                        css_class: Some("opaque"),
                    },
                    Var::Path(target.into()),
                ];
                if options.overwrite {
                    vars.push(Var::String {
                        label: "(overwrite)".into(),
                        css_class: Some("opaque"),
                    });
                }
                vars
            }
            Event::Idle => vec![Var::String {
                label: "Idle".into(),
                css_class: Some("bold"),
            }],
        }
    }
    pub fn copy() -> Self {
        Event::Copy {
            tag: common::never(),
            target: home::home_dir().unwrap(),
            options: CopyOptions { overwrite: false },
        }
    }
    pub fn mv() -> Self {
        Event::Move {
            tag: common::never(),
            target: home::home_dir().unwrap(),
            options: MoveOptions { overwrite: false },
        }
    }
    pub fn set_path(&mut self, p: PathBuf) {
        match self {
            Event::Copy {
                tag,
                target,
                options,
            } => *target = p,
            Event::Move {
                tag,
                target,
                options,
            } => *target = p,
            Event::Idle => {}
        }
    }
    pub fn set_tag(&mut self, t: Tag) {
        match self {
            Event::Copy {
                tag,
                target,
                options,
            } => *tag = t,
            Event::Move {
                tag,
                target,
                options,
            } => *tag = t,
            Event::Idle => {}
        }
    }
}

pub enum Var {
    String {
        label: String,
        css_class: Option<&'static str>,
    },
    Tag(Tag),
    Path(PathBuf),
}
