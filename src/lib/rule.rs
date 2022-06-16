//! Data structures and utilities related to the rule system.
use super::Event;
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
    pub fn new() -> Self {
        Self::default()
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
