use crate::lib::Event;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Log(Vec<LogEntry>);

impl Log {
    pub fn push(&mut self, entry: LogEntry) {
        self.0.push(entry);
    }
    pub fn entries(&self) -> &[LogEntry] {
        &self.0
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LogEntry {
    event: Event,
    source: PathBuf,
    file: PathBuf,
    time: DateTime<Local>,
}

impl LogEntry {
    pub fn new(event: &Event, source: impl AsRef<Path>, file: impl AsRef<Path>) -> Self {
        LogEntry {
            event: event.clone(),
            source: source.as_ref().to_owned(),
            file: file.as_ref().to_owned(),
            time: Local::now(),
        }
    }

    /// Get a reference to the log entry's event.
    pub fn event(&self) -> &Event {
        &self.event
    }

    /// Get a reference to the log entry's source.
    pub fn source(&self) -> &PathBuf {
        &self.source
    }

    /// Get a reference to the log entry's file.
    pub fn file(&self) -> &PathBuf {
        &self.file
    }

    /// Get a reference to the log entry's time.
    pub fn time(&self) -> DateTime<Local> {
        self.time
    }
}
