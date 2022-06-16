use super::TagExpr;
use crate::{fs::read_path, log::LogEntry};
use fs_extra::dir::CopyOptions;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    expr: TagExpr,
    tp: EventType,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventType {
    Copy { target: PathBuf, overwrite: bool },
    Move { target: PathBuf, overwrite: bool },
    Trash,
}

impl Event {
    pub fn name(&self) -> &str {
        match &self.tp {
            EventType::Copy { .. } => "Copy",
            EventType::Move { .. } => "Move",
            EventType::Trash => "Trash",
        }
    }
    pub fn icon_name(&self) -> &str {
        match &self.tp {
            EventType::Copy { .. } => "edit-copy-symbolic",
            EventType::Move { .. } => "go-jump-symbolic",
            EventType::Trash => "user-trash-symbolic",
        }
    }
    pub fn vars(&self) -> Vec<Var> {
        match &self.tp {
            EventType::Copy { target, overwrite } => {
                let mut vars = vec![
                    Var::String {
                        label: "Copy".into(),
                        css_class: Some("bold"),
                    },
                    Var::TagExpr(self.expr.clone()),
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
            EventType::Move { target, overwrite } => {
                let mut vars = vec![
                    Var::String {
                        label: "Move".into(),
                        css_class: Some("bold"),
                    },
                    Var::TagExpr(self.expr.clone()),
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
            EventType::Trash => vec![
                Var::String {
                    label: "Trash".into(),
                    css_class: Some("bold"),
                },
                Var::TagExpr(self.expr.clone()),
            ],
        }
    }
    pub fn copy() -> Self {
        Event {
            expr: TagExpr::default(),
            tp: EventType::Copy {
                target: dirs::home_dir().unwrap(),
                overwrite: false,
            },
        }
    }
    pub fn mv() -> Self {
        Event {
            expr: TagExpr::default(),
            tp: EventType::Move {
                target: dirs::home_dir().unwrap(),
                overwrite: false,
            },
        }
    }
    pub fn trash() -> Self {
        Event {
            expr: TagExpr::default(),
            tp: EventType::Trash,
        }
    }
    pub fn set_path(&mut self, p: PathBuf) {
        match &mut self.tp {
            EventType::Copy { target, .. } => *target = p,
            EventType::Move { target, .. } => *target = p,
            EventType::Trash => unreachable!(),
        }
    }
    pub fn tag_expr(&self) -> &TagExpr {
        &self.expr
    }
    pub fn tag_expr_mut(&mut self) -> &mut TagExpr {
        &mut self.expr
    }
    pub fn execute(
        &self,
        path: impl AsRef<Path>,
    ) -> anyhow::Result<Vec<SkippableResult<LogEntry>>> {
        let items = read_path(path)?;
        let files = items
            .into_iter()
            .filter_map(|mut item| {
                if let Ok(is) = self.tag_expr().is(&mut item) {
                    is
                } else {
                    false
                }
                .then(|| item)
            })
            .map(|item| item.path().to_owned())
            .collect::<Vec<_>>();
        let (target, results) = match &self.tp {
            EventType::Copy { target, overwrite } => {
                (Some(target), copy(&files, target, *overwrite))
            }
            EventType::Move { target, overwrite } => (Some(target), mv(&files, target, *overwrite)),
            EventType::Trash => (None, trash(&files)),
        };
        let results = results
            .into_iter()
            .map(|result| match result {
                SkippableResult::Ok(file) => SkippableResult::Ok(LogEntry::new(self, target, file)),
                SkippableResult::Skipped => SkippableResult::Skipped,
                SkippableResult::Err(e) => SkippableResult::Err(e),
            })
            .collect::<Vec<_>>();
        Ok(results)
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
    let to = to.as_ref();
    files
        .iter()
        .map(|file| {
            let path = file.as_ref();
            if let Some(file_name) = path.file_name() {
                if to.is_dir() {
                    if !overwrite && to.join(file_name).exists() {
                        SkippableResult::Skipped
                    } else {
                        let options = CopyOptions {
                            overwrite,
                            ..CopyOptions::new()
                        };
                        match fs_extra::copy_items(&[path], to, &options) {
                            Ok(_) => SkippableResult::Ok(path.to_owned()),
                            Err(e) => SkippableResult::Err(e.into()),
                        }
                    }
                } else {
                    SkippableResult::Err(anyhow::anyhow!("{path:?} is not a directory"))
                }
            } else {
                SkippableResult::Err(anyhow::anyhow!("{path:?} has no file name"))
            }
        })
        .collect()
}

fn mv(
    // Files to move
    files: &[impl AsRef<Path>],
    // Folder to move files into
    to: impl AsRef<Path>,
    overwrite: bool,
) -> Vec<SkippableResult<PathBuf>> {
    let to = to.as_ref();
    files
        .iter()
        .map(|file| {
            let path = file.as_ref();
            if let Some(file_name) = path.file_name() {
                if to.is_dir() {
                    if !overwrite && to.join(file_name).exists() {
                        SkippableResult::Skipped
                    } else {
                        let options = CopyOptions {
                            overwrite,
                            ..CopyOptions::new()
                        };
                        match fs_extra::move_items(&[path], to, &options) {
                            Ok(_) => SkippableResult::Ok(path.to_owned()),
                            Err(e) => SkippableResult::Err(e.into()),
                        }
                    }
                } else {
                    SkippableResult::Err(anyhow::anyhow!("{path:?} is not a directory"))
                }
            } else {
                SkippableResult::Err(anyhow::anyhow!("{path:?} has no file name"))
            }
        })
        .collect()
}

fn trash(
    // Files to remove
    files: &[impl AsRef<Path>],
) -> Vec<SkippableResult<PathBuf>> {
    files
        .iter()
        .map(|file| {
            let path = file.as_ref();
            if path.exists() {
                match trash::delete(path) {
                    Ok(_) => SkippableResult::Ok(path.to_owned()),
                    Err(e) => SkippableResult::Err(e.into()),
                }
            } else {
                SkippableResult::Skipped
            }
        })
        .collect()
}

#[derive(Debug)]
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

#[cfg(test)]
mod tests {
    use crate::lib::SkippableResult;

    use super::{copy, mv, trash};
    use std::path::PathBuf;

    fn test_dir_a() -> PathBuf {
        let dir = dirs::desktop_dir().unwrap().join("event-test-a");
        if !dir.exists() {
            std::fs::create_dir(&dir).unwrap();
        }
        dir
    }

    fn test_dir_b() -> PathBuf {
        let dir = dirs::desktop_dir().unwrap().join("event-test-b");
        if !dir.exists() {
            std::fs::create_dir(&dir).unwrap();
        }
        dir
    }

    #[test]
    fn copy_one() {
        let from = test_dir_a().join("test1.txt");
        if !from.exists() {
            std::fs::File::create(&from).unwrap();
        }
        let to = test_dir_b();
        if to.join("test1.txt").exists() {
            std::fs::remove_file(&to.join("test1.txt")).unwrap();
        }
        let result = copy(&[&from], to, false);
        assert!(from.exists());
        assert!(matches!(&result[..], &[SkippableResult::Ok(_)]));
    }

    #[test]
    fn copy_skip() {
        let from = test_dir_a().join("test2.txt");
        if !from.exists() {
            std::fs::File::create(&from).unwrap();
        }
        let to = test_dir_b();
        if !to.join("test2.txt").exists() {
            std::fs::File::create(&to.join("test2.txt")).unwrap();
        }
        let result = copy(&[&from], to, false);
        assert!(from.exists());
        assert!(matches!(&result[..], &[SkippableResult::Skipped]));
    }

    #[test]
    fn copy_overwrite() {
        let from = test_dir_a().join("test3.txt");
        if !from.exists() {
            std::fs::File::create(&from).unwrap();
        }
        let to = test_dir_b();
        if !to.join("test3.txt").exists() {
            std::fs::File::create(&to.join("test3.txt")).unwrap();
        }
        let result = copy(&[&from], to, true);
        assert!(from.exists());
        assert!(matches!(&result[..], &[SkippableResult::Ok(_)]));
    }

    #[test]
    fn copy_multiple() {
        let from1 = test_dir_a().join("test4-1.txt");
        if !from1.exists() {
            std::fs::File::create(&from1).unwrap();
        }
        let from2 = test_dir_a().join("test4-2.txt");
        if !from2.exists() {
            std::fs::File::create(&from2).unwrap();
        }
        let to = test_dir_b();
        if !to.join("test4-1.txt").exists() {
            std::fs::File::create(&to.join("test4-1.txt")).unwrap();
        }
        if to.join("test4-2.txt").exists() {
            std::fs::remove_file(&to.join("test4-2.txt")).unwrap();
        }
        let result = copy(&[&from1, &from2], to, false);
        assert!(from1.exists());
        assert!(from2.exists());
        assert!(matches!(
            &result[..],
            &[SkippableResult::Skipped, SkippableResult::Ok(_)]
        ));
    }

    #[test]
    fn mv_one() {
        let from = test_dir_a().join("test5.txt");
        if !from.exists() {
            std::fs::File::create(&from).unwrap();
        }
        let to = test_dir_b();
        if to.join("test5.txt").exists() {
            std::fs::remove_file(&to.join("test5.txt")).unwrap();
        }
        let result = mv(&[&from], to, false);
        assert!(!from.exists());
        assert!(matches!(&result[..], &[SkippableResult::Ok(_)]));
    }

    #[test]
    fn mv_skip() {
        let from = test_dir_a().join("test6.txt");
        if !from.exists() {
            std::fs::File::create(&from).unwrap();
        }
        let to = test_dir_b();
        if !to.join("test6.txt").exists() {
            std::fs::File::create(&to.join("test6.txt")).unwrap();
        }
        let result = mv(&[&from], to, false);
        assert!(from.exists());
        assert!(matches!(&result[..], &[SkippableResult::Skipped]));
    }

    #[test]
    fn mv_overwrite() {
        let from = test_dir_a().join("test7.txt");
        if !from.exists() {
            std::fs::File::create(&from).unwrap();
        }
        let to = test_dir_b();
        if !to.join("test7.txt").exists() {
            std::fs::File::create(&to.join("test7.txt")).unwrap();
        }
        let result = mv(&[&from], to, true);
        assert!(!from.exists());
        assert!(matches!(&result[..], &[SkippableResult::Ok(_)]));
    }

    #[test]
    fn mv_multiple() {
        let from1 = test_dir_a().join("test8-1.txt");
        if !from1.exists() {
            std::fs::File::create(&from1).unwrap();
        }
        let from2 = test_dir_a().join("test8-2.txt");
        if !from2.exists() {
            std::fs::File::create(&from2).unwrap();
        }
        let to = test_dir_b();
        if !to.join("test8-1.txt").exists() {
            std::fs::File::create(&to.join("test8-1.txt")).unwrap();
        }
        if to.join("test8-2.txt").exists() {
            std::fs::remove_file(&to.join("test8-2.txt")).unwrap();
        }
        let result = mv(&[&from1, &from2], to, false);
        assert!(from1.exists());
        assert!(!from2.exists());
        assert!(matches!(
            &result[..],
            &[SkippableResult::Skipped, SkippableResult::Ok(_)]
        ));
    }

    #[test]
    fn trash_one() {
        let file = test_dir_a().join("test9.txt");
        if !file.exists() {
            std::fs::File::create(&file).unwrap();
        }
        let result = trash(&[&file]);
        assert!(!file.exists());
        assert!(matches!(&result[..], &[SkippableResult::Ok(_)]));
    }

    #[test]
    fn trash_multiple() {
        let file1 = test_dir_a().join("test10-1.txt");
        if !file1.exists() {
            std::fs::File::create(&file1).unwrap();
        }
        let file2 = test_dir_a().join("test10-2.txt");
        if !file2.exists() {
            std::fs::File::create(&file2).unwrap();
        }
        let result = trash(&[&file1, &file2]);
        assert!(!file1.exists());
        assert!(!file2.exists());
        assert!(matches!(
            &result[..],
            &[SkippableResult::Ok(_), SkippableResult::Ok(_)]
        ));
    }
}
