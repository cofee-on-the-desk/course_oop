use serde::{Deserialize, Serialize};

use crate::{
    lib::{Event, Rule, Tag},
    log::Log,
};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

// TODO: don't actually serialize the whole struct, but use different fields separately, in different files
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Database {
    rules: HashMap<PathBuf, Vec<Rule>>,
    log: Arc<Mutex<Log>>,
}

impl Database {
    /// Get a reference to the database rules.
    pub fn rules(&self) -> &HashMap<PathBuf, Vec<Rule>> {
        &self.rules
    }

    /// Get a mutable reference to the database rules.
    pub fn rules_mut(&mut self) -> &mut HashMap<PathBuf, Vec<Rule>> {
        &mut self.rules
    }

    pub fn load() -> Self {
        let mut path = dirs::config_dir().expect("Unable to find application config directory");
        path.push("course_oop");
        path.push("db.conf");
        if !path.exists() {
            return Database::default();
        }
        let bytes =
            std::fs::read(&path).unwrap_or_else(|e| panic!("Unable to read file at {path:?}: {e}"));
        serde_json::from_slice(&bytes).expect("Unable to deserialize database")
    }

    pub fn save(&self) {
        let mut path = dirs::config_dir().expect("Unable to find application config directory");
        path.push("course_oop");
        if !path.exists() {
            std::fs::create_dir(&path);
        }
        path.push("db.conf");
        if !path.exists() {
            std::fs::File::create(&path);
        }
        let bits = serde_json::to_vec(self).expect("Unable to serialize database");
        std::fs::write(path, bits);
    }

    pub fn log(&self) -> &Arc<Mutex<Log>> {
        &self.log
    }
}

impl Default for Database {
    fn default() -> Self {
        let rules = HashMap::new();
        let log = Arc::new(Mutex::new(Log::default()));
        Database { rules, log }
    }
}
