use anyhow::Context;

use crate::{lib::Rule, log::Log};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

#[derive(Clone, Debug)]
pub struct Database {
    rules: HashMap<PathBuf, Vec<Rule>>,
    log: Arc<Mutex<Log>>,
}

const BASE_DIR_FILENAME: &str = "course_oop";
const RULES_FILENAME: &str = "rules.json";
const LOG_FILENAME: &str = "log.json";

impl Database {
    pub fn rules(&self) -> &HashMap<PathBuf, Vec<Rule>> {
        &self.rules
    }
    pub fn rules_mut(&mut self) -> &mut HashMap<PathBuf, Vec<Rule>> {
        &mut self.rules
    }
    pub fn log(&self) -> &Arc<Mutex<Log>> {
        &self.log
    }
    pub fn load() -> anyhow::Result<Self> {
        let base_dir = dirs::config_dir()
            .with_context(|| "Unable to find application config directory")?
            .join(BASE_DIR_FILENAME);

        if !base_dir.exists() {
            std::fs::create_dir(&base_dir)?;
        }

        let rules_path = base_dir.join(RULES_FILENAME);
        let log_path = base_dir.join(LOG_FILENAME);

        let rules = if rules_path.exists() {
            let rule_bytes = std::fs::read(&rules_path)?;
            serde_json::from_slice(&rule_bytes)?
        } else {
            HashMap::new()
        };
        let log = if log_path.exists() {
            let log_bytes = std::fs::read(&log_path)?;
            serde_json::from_slice(&log_bytes)?
        } else {
            Arc::new(Mutex::new(Log::new()))
        };

        Ok(Database { rules, log })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let base_dir = dirs::config_dir()
            .with_context(|| "Unable to find application config directory")?
            .join(BASE_DIR_FILENAME);

        if !base_dir.exists() {
            std::fs::create_dir(&base_dir)?;
        }

        let rules_path = base_dir.join(RULES_FILENAME);
        if !rules_path.exists() {
            std::fs::File::create(&rules_path)?;
        }

        let log_path = base_dir.join(LOG_FILENAME);
        if !log_path.exists() {
            std::fs::File::create(&log_path)?;
        }

        let rules_bits = serde_json::to_vec(&self.rules)?;
        let log_bits = serde_json::to_vec(&self.log)?;
        std::fs::write(rules_path, rules_bits)?;
        std::fs::write(log_path, log_bits)?;

        Ok(())
    }
}

impl Default for Database {
    fn default() -> Self {
        let rules = HashMap::new();
        let log = Arc::new(Mutex::new(Log::default()));
        Database { rules, log }
    }
}
