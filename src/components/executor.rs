use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        mpsc::{channel, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use crate::lib::SkippableResult;
use crate::{lib::Rule, log::Log};

struct StopMessage;

pub struct Executor {
    log: Arc<Mutex<Log>>,
    sender: Option<Sender<StopMessage>>,
}

impl Executor {
    pub fn new(log: &Arc<Mutex<Log>>) -> Self {
        Executor {
            log: log.clone(),
            sender: None,
        }
    }
    pub fn restart(&mut self, rule_map: &HashMap<PathBuf, Vec<Rule>>) {
        if let Some(sender) = self.sender.take() {
            sender
                .send(StopMessage)
                .expect("Unable to send stop message");
        }
        let (sender, receiver) = channel();
        self.sender = Some(sender);

        let rule_map = rule_map.clone();
        let log = self.log.clone();
        thread::spawn(move || loop {
            if receiver.try_recv().is_ok() {
                return;
            }
            for (dir, rules) in rule_map.iter() {
                for rule in rules {
                    for event in rule.events() {
                        match event.execute(dir) {
                            Ok(results) => {
                                let mut log = log.lock().expect("unable to aquire mutex");
                                for result in results {
                                    match result {
                                        SkippableResult::Ok(entry) => log.push(entry),
                                        SkippableResult::Err(e) => eprintln!("An error has occured while trying to execute the event on one of the items: {e}"),
                                        SkippableResult::Skipped => {}
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "An error has occured while trying to perform an event: {e}"
                                );
                            }
                        }
                    }
                }
            }
            std::thread::sleep(Duration::from_secs(15));
        });
    }

    pub fn stop(&mut self) {
        if let Some(sender) = self.sender.take() {
            sender
                .send(StopMessage)
                .expect("Unable to send stop message");
        }
    }
}
