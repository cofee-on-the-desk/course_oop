//! Data structures and utilities related to the rule system.
use crate::{fs::read_path, lib::tag::Tag, log::LogEntry};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Rule {
    title: String,
    events: Vec<Event>,
}

impl Default for Rule {
    fn default() -> Self {
        Rule {
            title: "New Rule".into(),
            events: Vec::new(),
        }
    }
}

impl Rule {
    pub fn new(condition: Tag, events: Vec<Event>) -> Self {
        Rule {
            title: "New Rule".into(),
            events,
        }
    }
    pub fn events(&self) -> &[Event] {
        &self.events[..]
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

use std::path::{Path, PathBuf};

use super::TagExpr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Copy {
        expr: TagExpr,
        target: PathBuf,
        overwrite: bool,
    },
    Move {
        expr: TagExpr,
        target: PathBuf,
        overwrite: bool,
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
                expr,
                target,
                overwrite,
            } => {
                let mut vars = vec![
                    Var::String {
                        label: "Copy".into(),
                        css_class: Some("bold"),
                    },
                    Var::TagExpr(expr.clone()),
                    Var::String {
                        label: "to".into(),
                        css_class: Some("opaque"),
                    },
                    Var::Path(target.into()),
                ];
                if *overwrite {
                    vars.push(Var::String {
                        label: "(overwrite)".into(),
                        css_class: Some("opaque"),
                    });
                }
                vars
            }
            Event::Move {
                expr,
                target,
                overwrite,
            } => {
                let mut vars = vec![
                    Var::String {
                        label: "Move".into(),
                        css_class: Some("bold"),
                    },
                    Var::TagExpr(expr.clone()),
                    Var::String {
                        label: "to".into(),
                        css_class: Some("opaque"),
                    },
                    Var::Path(target.into()),
                ];
                if *overwrite {
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
            expr: TagExpr::default(),
            target: home::home_dir().unwrap(),
            overwrite: false,
        }
    }
    pub fn mv() -> Self {
        Event::Move {
            expr: TagExpr::default(),
            target: home::home_dir().unwrap(),
            overwrite: false,
        }
    }
    pub fn set_path(&mut self, p: PathBuf) {
        match self {
            Event::Copy { target, .. } => *target = p,
            Event::Move { target, .. } => *target = p,
            Event::Idle => {}
        }
    }
    pub fn tag_expr(&self) -> &TagExpr {
        match self {
            Event::Copy { expr, .. } => expr,
            Event::Move { expr, .. } => expr,
            Event::Idle => unimplemented!(),
        }
    }
    pub fn tag_expr_mut(&mut self) -> &mut TagExpr {
        match self {
            Event::Copy { expr, .. } => expr,
            Event::Move { expr, .. } => expr,
            Event::Idle => unimplemented!(),
        }
    }
    pub fn execute(
        &self,
        path: impl AsRef<Path>,
    ) -> anyhow::Result<Vec<SkippableResult<LogEntry>>> {
        match self {
            Event::Copy {
                expr,
                target,
                overwrite,
            } => {
                let files = read_path(path)?
                    .into_iter()
                    .filter(|item| {
                        if let Ok(is) = expr.is(item.path()) {
                            is
                        } else {
                            false
                        }
                    })
                    .map(|item| item.path().to_owned())
                    .collect::<Vec<_>>();
                let results = copy(&files, target, *overwrite)
                    .into_iter()
                    .map(|result| match result {
                        SkippableResult::Ok(file) => {
                            SkippableResult::Ok(LogEntry::new(self, target, file))
                        }
                        SkippableResult::Skipped => SkippableResult::Skipped,
                        SkippableResult::Err(e) => SkippableResult::Err(e),
                    })
                    .collect::<Vec<_>>();
                Ok(results)
            }
            Event::Move {
                expr,
                target,
                overwrite,
            } => todo!(),
            Event::Idle => todo!(),
        }
    }
}

pub enum Var {
    String {
        label: String,
        css_class: Option<&'static str>,
    },
    TagExpr(TagExpr),
    Path(PathBuf),
}

fn copy(
    // Files to copy
    files: &[impl AsRef<Path>],
    // Folder to copy files into
    to: impl AsRef<Path>,
    overwrite: bool,
) -> Vec<SkippableResult<PathBuf>> {
    files
        .iter()
        .map(|file| {
            let path = file.as_ref();

            if let Some(file_name) = path.file_name() {
                let mut to = to.as_ref().to_owned();
                to.push(file_name);
                if !overwrite && to.exists() {
                    SkippableResult::Skipped
                } else {
                    let options = fs_extra::dir::CopyOptions::new();
                    match fs_extra::copy_items(&[path], to, &options) {
                        Ok(_) => SkippableResult::Ok(path.to_owned()),
                        Err(e) => SkippableResult::Err(e.into()),
                    }
                }
            } else {
                SkippableResult::Err(anyhow::anyhow!("File at path {path:?} has no file name"))
            }
        })
        .collect()
}

fn mv(
    path: impl AsRef<Path>,
    target: impl AsRef<Path>,
    overwrite: bool,
) -> anyhow::Result<LogEntry> {
    todo!()
}

pub enum SkippableResult<T> {
    Ok(T),
    Skipped,
    Err(anyhow::Error),
}

impl<T, E: std::error::Error + Send + Sync + 'static> From<Result<T, E>> for SkippableResult<T> {
    fn from(result: Result<T, E>) -> Self {
        match result {
            Ok(val) => SkippableResult::Ok(val),
            Err(e) => SkippableResult::Err(e.into()),
        }
    }
}
