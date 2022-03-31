//! Data structures and utilities related to the rule system.
use crate::lib::tag::Tag;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Rule {
    condition: Tag,
    events: Vec<Event>,
}

impl Rule {
    pub fn new(condition: Tag, events: Vec<Event>) -> Self {
        Rule { condition, events }
    }
    pub fn condition(&self) -> &Tag {
        &self.condition
    }
    pub fn events(&self) -> &[Event] {
        &self.events[..]
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
}
