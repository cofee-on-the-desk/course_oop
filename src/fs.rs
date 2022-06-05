use crate::lib::{InputWrapper, Item, ItemType};
use std::{
    cmp::Ordering,
    path::{Path, PathBuf},
};

pub struct Explorer {
    dir: Item,
    items: Vec<Item>,
    history: NavigationHistory,
}

impl Explorer {
    /// Get a reference to the explorer dir.
    pub fn dir(&self) -> &Item {
        &self.dir
    }

    /// Get a reference to the explorer items.
    pub fn items(&self) -> &[Item] {
        self.items.as_ref()
    }

    /// Get a reference to the explorer history.
    pub fn history(&self) -> &NavigationHistory {
        &self.history
    }

    pub fn open(&mut self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        self.update(&path, true)
    }

    pub fn go_back(&mut self) -> anyhow::Result<()> {
        let path = self.history.back().to_owned();
        self.update(&path, false)
    }

    pub fn go_forward(&mut self) -> anyhow::Result<()> {
        let path = self.history.forward().to_owned();
        self.update(&path, false)
    }

    pub fn refresh(&mut self) -> anyhow::Result<()> {
        let path = self.dir.path().to_owned();
        self.update(&path, false)
    }

    fn update(&mut self, path: impl AsRef<Path>, update_history: bool) -> anyhow::Result<()> {
        let path = path.as_ref();

        let dir = Item::try_from(InputWrapper(path))?;
        let items = read_path(path)?;

        if update_history {
            self.history.push(path);
        }

        self.dir = dir;
        self.items = items;

        Ok(())
    }
}

impl Default for Explorer {
    fn default() -> Self {
        let dir = Item::try_from(InputWrapper(
            home::home_dir().expect("Unable to find user home directory."),
        ))
        .expect("Unable to create read the user home directory.");

        assert_eq!(dir.tp(), &ItemType::Dir);

        let items = read_path(&dir.path()).unwrap_or_default();

        let history = NavigationHistory::new(dir.path());

        Explorer {
            dir,
            items,
            history,
        }
    }
}

#[derive(Debug)]
pub struct NavigationHistory {
    vec: Vec<PathBuf>,
    index: usize,
}

impl NavigationHistory {
    pub fn new(path: impl AsRef<Path>) -> Self {
        NavigationHistory {
            vec: vec![path.as_ref().to_owned()],
            index: 0,
        }
    }
    pub fn push(&mut self, path: impl AsRef<Path>) {
        self.vec.truncate(self.index + 1);
        self.vec.push(path.as_ref().to_owned());
        self.index += 1;
    }
    pub fn can_go_back(&self) -> bool {
        self.index > 0
    }
    pub fn can_go_forward(&self) -> bool {
        self.index + 1 < self.vec.len()
    }
    pub fn back(&mut self) -> &Path {
        if self.can_go_back() {
            self.index -= 1;
        }
        &self.vec[self.index]
    }
    pub fn forward(&mut self) -> &Path {
        if self.can_go_forward() {
            self.index += 1;
        }
        &self.vec[self.index]
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn len(&self) -> usize {
        self.vec.len()
    }
}

pub fn read_path(path: impl AsRef<Path>) -> anyhow::Result<Vec<Item>> {
    let mut items = std::fs::read_dir(path)?
        .filter_map(|res| res.ok())
        .filter_map(|entry| Item::try_from(InputWrapper(entry.path())).ok())
        .collect::<Vec<_>>();

    // Order items by name, folders first
    items.sort_by(|a, b| match (a.tp(), b.tp()) {
        (ItemType::Dir, ItemType::Dir) => a.name().cmp(&b.name()),
        (ItemType::Dir, _) => Ordering::Less,
        (_, ItemType::Dir) => Ordering::Greater,
        _ => a.name().cmp(&b.name()),
    });

    Ok(items)
}
