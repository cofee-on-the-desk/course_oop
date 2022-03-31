use crate::{App, AppMsg};
use relm4::ComponentSender;

/// Allows using functions (closures) as methods.
///
/// Can be useful inside the `view!` macro.
pub trait Bind {
    fn bind(&self, f: impl Fn(&Self)) {
        f(self)
    }
}

impl<T> Bind for T {}

/// An extension for the `anyhow::Result<()>`.
///
/// If the underlying value is `Err`, shows an error dialog.
pub trait Expect {
    fn or_show_error(self, description: &str, sender: &ComponentSender<App>);
}

impl Expect for anyhow::Result<()> {
    fn or_show_error(self, description: &str, sender: &ComponentSender<App>) {
        if let Err(e) = self {
            sender.input(AppMsg::Error(description.to_string(), e.to_string()));
        }
    }
}
