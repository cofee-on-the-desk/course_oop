use crate::lib::{common, CopyOptions, Event, MoveOptions, Rule, Tag};
use std::{collections::HashMap, path::PathBuf};

#[derive(Clone, Debug)]
pub struct Database {
    tags: Vec<Tag>,
    rules: HashMap<PathBuf, Vec<Rule>>,
}

impl Database {
    /// Get a reference to the database tags.
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }

    /// Get a reference to the database rules.
    pub fn rules(&self) -> &HashMap<PathBuf, Vec<Rule>> {
        &self.rules
    }

    /// Get a mutable reference to the database rules.
    pub fn rules_mut(&mut self) -> &mut HashMap<PathBuf, Vec<Rule>> {
        &mut self.rules
    }
}

impl Default for Database {
    fn default() -> Self {
        let tags = vec![
            common::file(),
            common::folder(),
            common::link(),
            common::item(),
            common::empty(),
            common::never(),
        ];
        let mut rules = HashMap::new();
        rules.insert(
            home::home_dir().unwrap(),
            vec![
                Rule::new(
                    common::item(),
                    vec![Event::Copy {
                        tag: common::item(),
                        target: "~/Images".into(),
                        options: CopyOptions::default(),
                    }],
                ),
                Rule::new(
                    common::item(),
                    vec![Event::Move {
                        tag: common::item(),
                        target: "~/Documents".into(),
                        options: MoveOptions::default(),
                    }],
                ),
            ],
        );
        Database { tags, rules }
    }
}
