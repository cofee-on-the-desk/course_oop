use super::SENDER;
use crate::AppMsg;

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
