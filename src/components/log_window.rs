//! A window for adding and editing rules.
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use gtk::prelude::{
    BoxExt, ButtonExt, EditableExt, EntryBufferExtManual, EntryExt, GtkWindowExt, OrientableExt,
    WidgetExt,
};
use relm4::{
    adw::{
        self,
        traits::{ActionRowExt, BinExt},
    },
    gtk::{self, prelude::IsA},
    view, ComponentParts, ComponentSender, RelmRemoveAllExt, Sender, SimpleComponent, WidgetPlus,
};

use crate::log::Log;
use crate::utils::Bind;
use crate::{
    lib::{Event, Rule, Tag, Var},
    log::LogEntry,
};

#[derive(Debug)]
pub struct LogWindow {
    log: Arc<Mutex<Log>>,
}

pub enum LogWindowInput {}

#[derive(Debug)]
pub enum LogWindowOutput {
    Save(Rule),
    Cancel,
    Delete,
}

#[relm4::component(pub)]
impl SimpleComponent for LogWindow {
    type Widgets = EditRuleWindowWidgets;

    type InitParams = Arc<Mutex<Log>>;

    type Input = LogWindowInput;
    type Output = LogWindowOutput;

    view! {
        root = gtk::Window {
            set_default_width: 780,
            set_default_height: 500,
            gtk::ScrolledWindow {
                set_hscrollbar_policy: gtk::PolicyType::Never,
                adw::Clamp {
                    set_maximum_size: 600,
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_margin_all: 15,
                        set_spacing: 5,
                        gtk::ListBox {
                            add_css_class: "boxed-list",
                            set_hexpand: true,
                            #[watch]
                            remove_all: (),
                            #[watch]
                            #[iterate]
                            append:
                            model.log
                                .lock()
                                .expect("unable to aquire mutex")
                                .entries()
                                .iter()
                                .rev()
                                .enumerate()
                                .map(|(index, entry)| entry_view(index, entry, &sender.input))
                                .collect::<Vec<_>>()
                                .iter(),
                        }
                    }
                }
            }

        }
    }

    fn init(
        log: Self::InitParams,
        root: &Self::Root,
        sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = LogWindow { log };
        let widgets = view_output!();
        widgets.root.present();
        ComponentParts { model, widgets }
    }
}

fn entry_view(
    index: usize,
    entry: &LogEntry,
    sender: &Sender<LogWindowInput>,
) -> impl IsA<gtk::Widget> {
    let time = entry.time();
    view! {
        row = gtk::ListBoxRow {
            gtk::CenterBox {
                set_margin_all: 10,
                set_start_widget = Some(&gtk::Box) {

                },
                set_end_widget = Some(&gtk::Box) {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 2,
                    gtk::Label {
                        set_label: &time.time().to_string(),
                    },
                    gtk::Label {
                        set_label: &time.date().to_string(),
                    },
                    add_css_class: "opaque",
                }
            }
        }
    }
    row
}
