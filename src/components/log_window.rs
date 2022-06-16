//! A window for adding and editing rules.
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use adw::prelude::BinExt;
use gtk::prelude::{BoxExt, ButtonExt, GtkWindowExt, IsA, OrientableExt, WidgetExt};
use relm4::{
    adw, gtk, view, ComponentParts, ComponentSender, RelmRemoveAllExt, SimpleComponent, WidgetPlus,
};

use crate::{lib::Event, log::Log};
use crate::{lib::Var, log::LogEntry};

#[derive(Debug)]
pub struct LogWindow {
    log: Arc<Mutex<Log>>,
}

pub enum LogWindowInput {
    Refresh,
}

#[derive(Debug)]
pub enum LogWindowOutput {}

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
            set_title: Some("Log of filesystem actions"),
            set_titlebar = Some(&gtk::HeaderBar) {
                pack_end = &gtk::Button {
                    set_icon_name: "view-refresh",
                    set_css_classes: &["flat", "circular"],
                    connect_clicked[sender] => move |_| { sender.input(LogWindowInput::Refresh); }
                }
            },
            gtk::ScrolledWindow {
                set_hscrollbar_policy: gtk::PolicyType::Never,
                adw::Clamp {
                    set_maximum_size: 800,
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
                                .map(entry_view)
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

fn entry_view(entry: &LogEntry) -> impl IsA<gtk::Widget> {
    let time = entry.time();
    view! {
        row = gtk::ListBoxRow {
            gtk::CenterBox {
                set_margin_all: 10,
                set_start_widget: Some(&event_view(entry.event(), entry.file())),
                set_end_widget = Some(&gtk::Box) {
                    set_margin_end: 15,
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    append?: &source_view(entry.source()),
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 2,
                        add_css_class: "opaque",
                        gtk::Label {
                            set_label: &time.time().format("%H:%M:%S").to_string(),
                        },
                        gtk::Label {
                            set_label: &time.date().format("%Y-%m-%d").to_string(),
                        },
                    }
                }
            }
        }
    }
    row
}

fn source_view(source: Option<&Path>) -> Option<impl IsA<gtk::Widget>> {
    if let Some(source) = source {
        let source_str = source.to_string_lossy();
        view! {
            gtk_box = gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                set_tooltip_text: Some(&source_str),
                set_hexpand: true,
                gtk::Image {
                    set_icon_name: Some("folder-symbolic"),
                },
                gtk::Label {
                    set_label: &source_str,
                    set_width_chars: 10,
                    set_lines: 1,
                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                },
            }
        }
        Some(gtk_box)
    } else {
        None
    }
}

fn var_view(var: &Var, path: &Path) -> impl IsA<gtk::Widget> {
    let bin = adw::Bin::new();
    match var {
        Var::String { label, css_class } => bin.set_child(Some(
            &gtk::Label::builder()
                .label(label)
                .css_classes(css_class.map_or_else(Vec::new, |class| vec![class.into()]))
                .build(),
        )),
        Var::TagExpr(_) => bin.set_child(Some(
            &gtk::Label::builder()
                .label(&path.file_name().expect("no filename").to_string_lossy())
                .max_width_chars(15)
                .lines(1)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .tooltip_text(&path.to_string_lossy())
                .build(),
        )),
        Var::Path(path) => bin.set_child(Some(
            &gtk::Button::builder()
                .label(&path.to_string_lossy())
                .css_classes(vec!["link".into()])
                .build(),
        )),
    }
    bin
}

fn event_view(event: &Event, path: &Path) -> impl IsA<gtk::Widget> {
    let vars = event
        .vars()
        .iter()
        .map(|event| var_view(event, path))
        .collect::<Vec<_>>();
    view! {
        container = gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 10,
            gtk::Image { set_icon_name: Some(event.icon_name()), },
            #[iterate]
            append: vars.iter(),
        }
    }
    container
}

// TODO: Cleanup the view functions, deduplicate
