use crate::AppMsg;
use relm4::Sender;
use std::sync::Mutex;

mod bind;
pub use bind::Bind;

mod expect;
pub use expect::Expect;

mod path;
pub use path::PathExt;

lazy_static::lazy_static! {
    /// Global message sender.
    /// Made for convenience so moving it between functions is easier.
    pub static ref SENDER: AppSender = AppSender::new();
}

pub struct AppSender(Mutex<Option<Sender<AppMsg>>>);

impl AppSender {
    pub fn new() -> Self {
        AppSender(Mutex::new(None))
    }
    pub fn init(&self, sender: &Sender<AppMsg>) {
        *self.0.lock().unwrap() = Some(sender.clone());
    }
    pub fn send(&self, msg: AppMsg) {
        if let Some(sender) = self.0.lock().unwrap().as_ref() {
            sender.send(msg);
        }
    }
}
