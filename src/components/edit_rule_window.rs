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

use crate::lib::Base;
use crate::lib::{all_tags_sorted_by_columns, Event, Rule, Tag, TagExpr, Var};
use crate::util::Bind;
use crate::AppMsg;
use crate::SENDER;

#[derive(Debug)]
pub struct EditRuleWindow {
    mode: EditMode,
    root: gtk::Window,
    rule: Rule,
    tag_select_multiple: Arc<Mutex<bool>>,
    tag_negate: Arc<Mutex<bool>>,
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
    AddEvent(Event),
    ClickedTag(usize, Tag),
    ResetTag(usize),
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
                            .map(|(index, rule)| row_view(index, rule, &sender.input, model.tag_select_multiple.clone(), model.tag_negate.clone()))
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
                                        connect_clicked[sender, popover] => move |_| { sender.input(EditRuleInput::AddEvent(Event::copy()) ); popover.hide() },
                                    },
                                    append = &icon_label_button("Move", "go-jump-symbolic") -> gtk::Button {
                                        connect_clicked[sender, popover] => move |_| { sender.input(EditRuleInput::AddEvent(Event::mv()) ); popover.hide() },
                                    },
                                    append = &icon_label_button("Trash", "user-trash-symbolic") -> gtk::Button {
                                        connect_clicked[sender, popover] => move |_| { sender.input(EditRuleInput::AddEvent(Event::trash()) ); popover.hide() },
                                    },
                                }
                            }
                        }
                    }
                }
            },
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
            tag_select_multiple: Arc::new(Mutex::new(false)),
            tag_negate: Arc::new(Mutex::new(false)),
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
            EditRuleInput::AddEvent(event) => {
                self.rule.events_mut().push(event);
            }
            EditRuleInput::ChangedPath(index, path) => {
                if let Some(event) = self.rule.events_mut().get_mut(index) {
                    event.set_path(path);
                }
            }
            EditRuleInput::ClickedTag(index, tag) => {
                if let Some(event) = self.rule.events_mut().get_mut(index) {
                    let mut tag_select_multiple = self.tag_select_multiple.lock().unwrap();
                    let mut tag_negate = self.tag_negate.lock().unwrap();
                    if event.tag_expr().has(&tag) {
                        event.tag_expr_mut().remove(&tag);
                    } else if *tag_select_multiple {
                        event.tag_expr_mut().push(tag, !*tag_negate);
                    } else {
                        *event.tag_expr_mut() = TagExpr::new(tag, !*tag_negate);
                    }
                    *tag_select_multiple = false;
                    *tag_negate = false;
                }
            }
            EditRuleInput::ResetTag(index) => {
                if let Some(event) = self.rule.events_mut().get_mut(index) {
                    let mut tag_select_multiple = self.tag_select_multiple.lock().unwrap();
                    let mut tag_negate = self.tag_negate.lock().unwrap();
                    *tag_select_multiple = false;
                    *tag_negate = false;
                    *event.tag_expr_mut() = TagExpr::default();
                }
            }
        }
    }
}

fn row_view(
    index: usize,
    event: &Event,
    sender: &Sender<EditRuleInput>,
    tag_select_multiple: Arc<Mutex<bool>>,
    tag_negate: Arc<Mutex<bool>>,
) -> impl IsA<gtk::Widget> {
    let row = adw::ActionRow::new();
    row.add_prefix(&event_view(
        index,
        event,
        sender,
        tag_select_multiple,
        tag_negate,
    ));

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
    tag_select_multiple: Arc<Mutex<bool>>,
    tag_negate: Arc<Mutex<bool>>,
) -> impl IsA<gtk::Widget> {
    let vars = event
        .vars()
        .iter()
        .map(|event| {
            var_view(
                index,
                event,
                sender,
                tag_select_multiple.clone(),
                tag_negate.clone(),
            )
        })
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

pub fn var_view(
    index: usize,
    var: &Var,
    sender: &Sender<EditRuleInput>,
    tag_select_multiple: Arc<Mutex<bool>>,
    tag_negate: Arc<Mutex<bool>>,
) -> impl IsA<gtk::Widget> {
    let bin = adw::Bin::new();
    match var {
        Var::String { label, css_class } => bin.set_child(Some(
            &gtk::Label::builder()
                .label(label)
                .css_classes(css_class.map_or_else(Vec::new, |class| vec![class.into()]))
                .build(),
        )),
        Var::TagExpr(expr) => bin.set_child(Some(&{
            let columns = all_tags_sorted_by_columns();
            view! {
                button = gtk::MenuButton {
                    set_margin_top: 12,
                    set_margin_bottom: 12,
                    set_label: &expr.name(),
                    set_tooltip_text: Some(&expr.desc()),
                    set_popover: popover = Some(&gtk::Popover) {
                        gtk::Box { set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 10,
                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_margin_start: 10,
                                set_margin_end: 10,
                                set_spacing: 15,
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 10,
                                    set_width_request: 300,
                                    gtk::Label { set_markup: "<b>Filetype</b>" },
                                    gtk::FlowBox {
                                        set_selection_mode: gtk::SelectionMode::None,
                                        #[iterate]
                                        insert[-1]: columns[0]
                                            .iter()
                                            .map(|tag| tag_view(index, expr, tag, sender, &popover))
                                            .collect::<Vec<_>>()
                                            .iter(),
                                    }
                                },
                                gtk::Separator {},
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 10,
                                    set_width_request: 300,
                                    gtk::Label { set_markup: "<b>Size</b>" },
                                    gtk::FlowBox {
                                        set_selection_mode: gtk::SelectionMode::None,
                                        #[iterate]
                                        insert[-1]: columns[1]
                                            .iter()
                                            .map(|tag| tag_view(index, expr, tag, sender, &popover))
                                            .collect::<Vec<_>>()
                                            .iter(),
                                    },
                                    gtk::Label { set_margin_start: 10, set_label: "Custom", set_xalign: 0. },
                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Horizontal,
                                        set_margin_all: 10,
                                        set_spacing: 10,
                                        append: custom_size = &gtk::Entry {
                                            set_hexpand: true,
                                            set_placeholder_text: Some("250MB"),
                                            connect_changed[custom_size_confirm] => move |entry| {
                                                if entry.buffer().text().trim().is_empty() {
                                                    custom_size_confirm.set_sensitive(false);
                                                }
                                                else {
                                                    custom_size_confirm.set_sensitive(true);
                                                }
                                            }
                                        },
                                        append: custom_size_confirm = &gtk::Button {
                                            set_sensitive: false,
                                            set_icon_name: "emblem-ok-symbolic",
                                            set_css_classes: &["flat", "circular"],
                                            connect_clicked[sender, custom_size, popover] => move |_| {
                                                let size = byte_unit::Byte::from_str(custom_size.buffer().text());
                                                match size {
                                                    Ok(size) => {
                                                        let size_str = size.get_appropriate_unit(true).to_string();
                                                        popover.hide();
                                                        sender.send(EditRuleInput::ClickedTag(index, Tag {
                                                            name: format!("💾 < {}", &size_str),
                                                            basis: Base::SizeLT(size),
                                                            desc: format!("A custom tag which includes files that are < {} in size.", &size_str),
                                                        }));
                                                    }
                                                    Err(e) => {
                                                        popover.hide();
                                                        SENDER.send(AppMsg::Error("Wrong file size formatting".to_string(), e.to_string()));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                },
                                gtk::Separator {},
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 10,
                                    set_width_request: 300,
                                    gtk::Label { set_markup: "<b>Creation date</b>" },
                                    gtk::FlowBox {
                                        set_selection_mode: gtk::SelectionMode::None,
                                        #[iterate]
                                        insert[-1]: columns[2]
                                        .iter()
                                        .map(|tag| tag_view(index, expr, tag, sender, &popover))
                                        .collect::<Vec<_>>()
                                        .iter(),
                                    },

                                    gtk::Label { set_margin_start: 10, set_label: "Custom", set_xalign: 0. },
                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Horizontal,
                                        set_margin_all: 10,
                                        set_spacing: 10,
                                        append: creation_date = &gtk::Entry {
                                            set_hexpand: true,
                                            set_placeholder_text: Some("48h"),
                                            connect_changed[creation_date_confirm] => move |entry| {
                                                if entry.buffer().text().trim().is_empty() {
                                                    creation_date_confirm.set_sensitive(false);
                                                }
                                                else {
                                                    creation_date_confirm.set_sensitive(true);
                                                }
                                            }
                                        },
                                        append: creation_date_confirm = &gtk::Button {
                                            set_sensitive: false,
                                            set_icon_name: "emblem-ok-symbolic",
                                            set_css_classes: &["flat", "circular"],
                                            connect_clicked[sender, creation_date, popover] => move |_| {
                                                let duration = duration_string::DurationString::try_from(creation_date.buffer().text());
                                                match duration {
                                                    Ok(duration) => {
                                                        popover.hide();
                                                        sender.send(EditRuleInput::ClickedTag(index, Tag {
                                                            name: format!("🕒 < {}", &duration),
                                                            desc: format!("A custom tag which includes files that were created more than {} ago.", &duration),
                                                            basis: Base::LifetimeGT(duration.into()),
                                                        }));
                                                    }
                                                    Err(e) => {
                                                        popover.hide();
                                                        SENDER.send(AppMsg::Error("Wrong file size formatting".to_string(), e));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                },
                                gtk::Separator {},
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 10,
                                    set_width_request: 300,
                                    gtk::Label { set_markup: "<b>Other</b>" },
                                    gtk::FlowBox {
                                        set_selection_mode: gtk::SelectionMode::None,
                                        #[iterate]
                                        insert[-1]: columns[3]
                                        .iter()
                                        .map(|tag| tag_view(index, expr, tag, sender, &popover))
                                        .collect::<Vec<_>>()
                                        .iter(),
                                    }
                                },
                            },
                            gtk::CenterBox {
                                set_margin_all: 10,
                                set_start_widget = Some(&gtk::Box) {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 10,
                                    gtk::Label { set_markup: "Use <b>Shift</b> to select multiple tags.", set_xalign: 0.  },
                                    gtk::Label { set_markup: "Use <b>Ctrl</b> to exclude a tag from the set.", set_xalign: 0. },
                                },
                                set_end_widget = Some(&gtk::Box) {
                                    gtk::Button {
                                        set_icon_name: "view-refresh",
                                        set_css_classes: &["circular", "flat"],
                                        connect_clicked[sender, popover] => move |_| {
                                            popover.hide();
                                            sender.send(EditRuleInput::ResetTag(index));
                                        }
                                    }
                                }
                            }
                        },
                        add_controller = &gtk::EventControllerKey {
                            connect_key_pressed[tag_select_multiple, tag_negate] => move |_, key, _, _| {
                                if key == gtk::gdk::Key::Shift_L || key == gtk::gdk::Key::Shift_R {
                                    if let Ok(mut b) = tag_select_multiple.lock() {
                                        *b = true;
                                    }
                                } else if key == gtk::gdk::Key::Control_L || key == gtk::gdk::Key::Control_R {
                                    if let Ok(mut b) = tag_negate.lock() {
                                        *b = true;
                                    }
                                }
                                gtk::Inhibit(false)
                            },
                            connect_key_released[tag_select_multiple, tag_negate] => move |_, key, _, _| {
                                if key == gtk::gdk::Key::Shift_L || key == gtk::gdk::Key::Shift_R {
                                    if let Ok(mut b) = tag_select_multiple.lock() {
                                        *b = false;
                                    }
                                } else if key == gtk::gdk::Key::KP_Space {
                                    if let Ok(mut b) = tag_negate.lock() {
                                        *b = false;
                                    }
                                }
                            }
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

fn tag_view(
    index: usize,
    expr: &TagExpr,
    tag: &Tag,
    sender: &Sender<EditRuleInput>,
    popover: &gtk::Popover,
) -> impl IsA<gtk::Widget> {
    view! {
        widget = gtk::Button {
            set_margin_top: 8,
            set_margin_bottom: 8,
            set_label: tag.name(),
            set_tooltip_text: Some(tag.desc()),
            add_css_class: "tag",
            add_css_class?: expr.has(tag).then(|| "opaque"),
            connect_clicked[sender, tag, popover] => move |_| {
                popover.hide();
                sender.send(EditRuleInput::ClickedTag(index, tag.clone()));
            }
        }
    }
    widget
}
