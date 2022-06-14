use crate::AppMsg;
use relm4::Sender;
use std::sync::Mutex;

/// Allows using functions (closures) as methods.
///
/// Can be useful inside the `view!` macro.
pub trait Bind {
    fn bind(&self, f: impl Fn(&Self)) {
        f(self)
    }
}

impl<T> Bind for T {}

/// Extension for `anyhow::Result<()>`.
///
/// If the underlying value is `Err`, shows an error dialog.
pub trait Expect {
    fn or_show_error(self, desc: &str);
}

impl Expect for anyhow::Result<()> {
    fn or_show_error(self, desc: &str) {
        if let Err(e) = self {
            SENDER.send(AppMsg::Error(desc.to_string(), e.to_string()));
        }
    }
}

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
