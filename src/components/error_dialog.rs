//! Dialog to display an error.
use relm4::{
    gtk::{
        self,
        prelude::{DialogExt, GtkWindowExt},
    },
    ComponentParts, ComponentSender, SimpleComponent,
};

pub struct ErrorDialog;

#[relm4::component(pub)]
impl SimpleComponent for ErrorDialog {
    type Widgets = ErrorDialogWidgets;

    type InitParams = (String, String);

    type Input = ();
    type Output = ();

    view! {
        dialog() -> gtk::MessageDialog {
            set_text: Some(&description),
            set_secondary_text: Some(&error_text),
            connect_response => |dialog, _| {
                dialog.destroy();
            }
        }
    }

    fn init(
        (description, error_text): Self::InitParams,
        root: &Self::Root,
        _sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let widgets = view_output!();
        root.present();
        ComponentParts {
            model: ErrorDialog,
            widgets,
        }
    }
}

fn dialog() -> gtk::MessageDialog {
    gtk::MessageDialog::builder()
        .buttons(gtk::ButtonsType::Close)
        .modal(true)
        .build()
}
