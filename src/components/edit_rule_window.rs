//! A window for adding and editing rules.
use std::path::PathBuf;

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

use crate::lib::common;
use crate::lib::{Event, Rule, Tag, Var};
use crate::utils::Bind;

#[derive(Debug)]
pub struct EditRuleWindow {
    mode: EditMode,
    root: gtk::Window,
    rule: Rule,
}

#[derive(Debug, PartialEq)]
pub enum EditMode {
    Create,
    Edit,
}

pub enum EditRuleInput {
    Save,
    Delete,
    SetTitle(String),
    RemoveEventAt(usize),
    AddEventCopy,
    AddEventMove,
    ChangedTag(usize, Tag),
    ChangedPath(usize, PathBuf),
}

#[derive(Debug)]
pub enum EditRuleOutput {
    Save(Rule),
    Cancel,
    Delete,
}

#[relm4::component(pub)]
impl SimpleComponent for EditRuleWindow {
    type Widgets = EditRuleWindowWidgets;

    type InitParams = (Rule, EditMode);

    type Input = EditRuleInput;
    type Output = EditRuleOutput;

    view! {
        root = gtk::Window {
            set_default_width: 780,
            set_default_height: 500,
            #[watch]
            set_title: Some(&match model.mode {
                EditMode::Create => {
                    format!("Create rule \"{}\"", model.rule.title())
                }
                EditMode::Edit => {
                    format!("Edit rule \"{}\"", model.rule.title())
                }
            }),
            set_modal: true,
            connect_close_request[sender] => move |_| {
                sender.output(EditRuleOutput::Cancel);
                gtk::Inhibit(false)
            },
            set_titlebar = Some(&gtk::HeaderBar) {
                pack_start = &icon_label_button("Save", "emblem-ok-symbolic") -> gtk::Button {
                    connect_clicked[sender] => move |_| {
                        sender.input(EditRuleInput::Save);
                    }
                },
                pack_end = &gtk::MenuButton {
                    set_icon_name: "view-more-symbolic",
                    set_popover: view_more_popover = Some(&gtk::Popover) {
                        gtk::Button {
                            set_label: "Delete",
                            connect_clicked[sender, view_more_popover] => move |_| {
                                view_more_popover.hide();
                                sender.input(EditRuleInput::Delete);
                            }
                        }
                    }
                }
            },
            gtk::ScrolledWindow {
                set_hscrollbar_policy: gtk::PolicyType::Never,
                adw::Clamp {
                    set_maximum_size: 600,
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_margin_all: 15,
                        set_spacing: 5,
                        gtk::Label { set_markup: "<b>Title</b>", set_xalign: 0. },
                        gtk::Entry {
                            set_hexpand: true,
                            connect_changed[sender] => move |entry| {
                                let title = entry.buffer().text();
                                sender.input(EditRuleInput::SetTitle(title));
                            },
                            bind: |entry| {
                                entry.buffer().set_text(model.rule.title());
                            }
                        },
                        gtk::Label { set_margin_top: 10, set_markup: "<b>Events</b>", set_xalign: 0. },
                        gtk::ListBox {
                            add_css_class: "boxed-list",
                            set_hexpand: true,
                            #[watch]
                            remove_all: (),
                            #[watch]
                            #[iterate]
                            append: model
                            .rule
                            .events()
                            .iter()
                            .enumerate()
                            .map(|(index, rule)| row_view(index, rule, &sender.input))
                            .collect::<Vec<_>>()
                            .iter(),
                        },
                        gtk::MenuButton {
                            set_label: "Add a new event",
                            set_popover: popover = Some(&gtk::Popover) {
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Horizontal,
                                    set_margin_all: 5,
                                    append = &icon_label_button("Copy", "edit-copy-symbolic") -> gtk::Button {
                                        connect_clicked[sender, popover] => move |_| { sender.input(EditRuleInput::AddEventCopy ); popover.hide() },
                                    },
                                    append = &icon_label_button("Move", "go-jump-symbolic") -> gtk::Button {
                                        connect_clicked[sender, popover] => move |_| { sender.input(EditRuleInput::AddEventMove ); popover.hide() },
                                    },
                                    append = &icon_label_button("Notify", "starred-symbolic") -> gtk::Button {
                                    //    connect_clicked[sender, popover] => move |_| { sender.input(EditRuleInput::AddEventNotify ) },
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn init(
        (rule, mode): Self::InitParams,
        root: &Self::Root,
        sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = EditRuleWindow {
            rule,
            root: root.clone(),
            mode,
        };
        let widgets = view_output!();
        widgets.root.present();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: &ComponentSender<Self>) {
        match message {
            EditRuleInput::Save => {
                sender.output(EditRuleOutput::Save(self.rule.clone()));
                self.root.destroy();
            }
            EditRuleInput::Delete => {
                // todo: show some warning
                sender.output(EditRuleOutput::Delete);
                self.root.destroy();
            }
            EditRuleInput::SetTitle(title) => {
                *self.rule.title_mut() = title;
            }
            EditRuleInput::RemoveEventAt(index) => {
                self.rule.events_mut().remove(index);
            }
            EditRuleInput::AddEventCopy => {
                self.rule.events_mut().push(Event::copy());
            }
            EditRuleInput::AddEventMove => {
                self.rule.events_mut().push(Event::mv());
            }
            EditRuleInput::ChangedPath(index, path) => {
                if let Some(event) = self.rule.events_mut().get_mut(index) {
                    event.set_path(path);
                }
            }
            EditRuleInput::ChangedTag(index, tag) => {
                if let Some(event) = self.rule.events_mut().get_mut(index) {
                    event.set_tag(tag);
                }
            }
        }
    }
}

fn row_view(index: usize, event: &Event, sender: &Sender<EditRuleInput>) -> impl IsA<gtk::Widget> {
    let row = adw::ActionRow::new();
    row.add_prefix(&event_view(index, event, sender));

    view! {
        remove_button = gtk::Button {
            set_icon_name: "list-remove-symbolic",
            add_css_class: "circular",
            set_margin_top: 15,
            set_margin_bottom: 15,
            connect_clicked[sender] => move |_| {
                sender.send(EditRuleInput::RemoveEventAt(index));
            }
        }
    }
    row.add_suffix(&remove_button);

    row
}

fn event_view(
    index: usize,
    event: &Event,
    sender: &Sender<EditRuleInput>,
) -> impl IsA<gtk::Widget> {
    let vars = event
        .vars()
        .iter()
        .map(|event| var_view(index, event, sender))
        .collect::<Vec<_>>();
    view! {
        container = gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 10,
            gtk::Image { set_icon_name: Some(event.gtk_icon()), },
            #[iterate]
            append: vars.iter(),
        }
    }
    container
}

fn icon_label_button(label: &str, icon: &str) -> gtk::Button {
    view! {
        button = gtk::Button {
            add_css_class: "flat",
            set_child = Some(&gtk::Box) {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
                gtk::Image {
                    set_icon_name: Some(icon),
                },
                gtk::Label {
                    set_label: label,
                }
            }
        }
    }
    button
}

pub fn var_view(index: usize, var: &Var, sender: &Sender<EditRuleInput>) -> impl IsA<gtk::Widget> {
    let bin = adw::Bin::new();
    match var {
        Var::String { label, css_class } => bin.set_child(Some(
            &gtk::Label::builder()
                .label(label)
                .css_classes(css_class.map_or_else(Vec::new, |class| vec![class.into()]))
                .build(),
        )),
        Var::Tag(tag) => bin.set_child(Some(&{
            view! {
                button = gtk::MenuButton {
                    set_margin_top: 12,
                    set_margin_bottom: 12,
                    set_label: &tag.emoji_name(),
                    set_tooltip_text?: Some(tag.desc()),
                    set_popover: popover = Some(&gtk::Popover) {
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_margin_start: 10,
                            set_margin_end: 10,
                            set_spacing: 15,
                            #[iterate]
                            append: common::all().into_iter().map(|t| {
                                view! {
                                    widget = gtk::Button {
                                        set_margin_top: 12,
                                        set_margin_bottom: 12,
                                        set_label: &t.emoji_name(),
                                        set_tooltip_text?: Some(t.desc()),
                                        add_css_class: "tag",
                                        set_sensitive: &t != tag,
                                        add_css_class?: (&t == tag).then(|| "opaque"),
                                        connect_clicked[sender, t, popover] => move |_| {
                                            popover.hide();
                                            sender.send(EditRuleInput::ChangedTag(index, t.clone()));
                                        }
                                    }
                                }
                                widget
                            }).collect::<Vec<_>>().iter(),
                        }
                    }
                }
            }
            button
        })),
        Var::Path(path) => bin.set_child(Some(&{
            view! {
                button = gtk::MenuButton {
                    set_margin_top: 10,
                    set_margin_bottom: 10,
                    set_label: &path.to_string_lossy(),
                    add_css_class: "link",
                    set_popover: popover = Some(&gtk::Popover) {
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 15,
                            append: entry = &gtk::Entry {
                                connect_changed[ok_button] => move |entry| {
                                    let text = entry.buffer().text();
                                    ok_button.set_sensitive(parse_path(&text).is_some());
                                },
                                bind: |entry| {
                                    entry.buffer().set_text(&path.to_string_lossy());
                                }
                            },
                            append: ok_button = &gtk::Button {
                                set_icon_name: "emblem-ok-symbolic",
                                add_css_class: "circular",
                                connect_clicked[sender, entry, popover] => move |_| {
                                    let text = entry.buffer().text();
                                    if let Some(path) = parse_path(&text) {
                                        popover.hide();
                                        sender.send(EditRuleInput::ChangedPath(index, path));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            button
        })),
    }
    bin
}

fn parse_path(s: &str) -> Option<PathBuf> {
    if s.is_empty() {
        return None;
    }

    let path = PathBuf::from(s);
    if path.is_absolute() || s.starts_with('~') {
        Some(path)
    } else {
        None
    }
}
