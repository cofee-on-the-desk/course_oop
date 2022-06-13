mod lib;

use components::edit_rule_window::{EditMode, EditRuleOutput, EditRuleWindow};
use components::executor::Executor;
use components::log_window::LogWindow;
use lib::{Event, Item, ItemType, Rule, Tag, Var};

mod db;
use db::Database;

mod fs;
use fs::Explorer;

mod components;
use components::error_dialog::ErrorDialog;

pub mod log;

mod utils;
use utils::Expect;

use adw::prelude::{BinExt, ExpanderRowExt};
use relm4::gtk::glib::FromVariant;
use relm4::gtk::prelude::{BoxExt, Cast, IsA, StaticType, StaticVariantType, ToVariant};
use relm4::{
    adw, component, gtk, view, Component, ComponentParts, ComponentSender, RelmApp,
    RelmRemoveAllExt, SimpleComponent, WidgetPlus,
};
use serde::{Deserialize, Serialize};

use gtk::prelude::{ButtonExt, GtkWindowExt, OrientableExt, WidgetExt};
use utils::SENDER;

use crate::lib::all_tags;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AppMsg {
    Error(String, String),
    GoBack,
    GoForward,
    OpenAt(usize),
    Refresh,
    NewRuleRequest,
    NewRule(Rule),
    EditRuleRequest(usize),
    EditRule(usize, Rule),
    DeleteRule(usize),
    ShowLog,
    Ignore,
    Quit,
}

impl StaticVariantType for AppMsg {
    fn static_variant_type() -> std::borrow::Cow<'static, gtk::glib::VariantTy> {
        <[u8]>::static_variant_type()
    }
}

impl FromVariant for AppMsg {
    fn from_variant(variant: &gtk::glib::Variant) -> Option<Self> {
        bincode::deserialize(&(<Vec<u8> as FromVariant>::from_variant(variant)?)).ok()
    }
}

impl ToVariant for AppMsg {
    fn to_variant(&self) -> gtk::glib::Variant {
        bincode::serialize(&self).unwrap().to_variant()
    }
}

pub struct AppData {
    pub explorer: Explorer,
    pub db: Database,
}

impl AppData {
    pub fn new(db: Database) -> Self {
        AppData {
            explorer: Explorer::default(),
            db,
        }
    }
    pub fn current_dir_rules(&self) -> Option<&[Rule]> {
        self.db
            .rules()
            .get(self.explorer.dir().path())
            .map(|v| v.as_slice())
    }
    pub fn current_dir_rules_mut(&mut self) -> Option<&mut Vec<Rule>> {
        self.db.rules_mut().get_mut(self.explorer.dir().path())
    }
}

pub struct App {
    pub data: AppData,
    pub executor: Executor,
    pub root: gtk::ApplicationWindow,
    pub is_active: bool,
}

#[component(pub)]
impl SimpleComponent for App {
    type Widgets = AppWidgets;

    type InitParams = Database;

    type Input = AppMsg;
    type Output = ();

    view! {
        window = gtk::ApplicationWindow {
            set_default_width: 960,
            set_default_height: 640,
            connect_close_request[sender] => move |_| {
                sender.input(AppMsg::Quit);
                gtk::Inhibit(true)
            },
            set_titlebar = Some(&gtk::HeaderBar) {
                pack_start = &gtk::Button::from_icon_name("go-previous") {
                    #[watch]
                    set_sensitive: model.data.explorer.history().can_go_back(),
                    connect_clicked[sender] => move |_| {
                        sender.input(AppMsg::GoBack);
                    }
                },
                pack_start = &gtk::Button::from_icon_name("go-next") {
                    #[watch]
                    set_sensitive: model.data.explorer.history().can_go_forward(),
                    connect_clicked[sender] => move |_| {
                        sender.input(AppMsg::GoForward);
                    }
                },
                pack_start = &gtk::Box {
                    set_margin_start: 5,
                    set_spacing: 10,
                    set_orientation: gtk::Orientation::Horizontal,
                    gtk::Image {
                        set_from_file: Some(ItemType::Dir.icon_path()),
                        set_icon_size: gtk::IconSize::Large,
                    },
                    gtk::Label {
                        #[watch]
                        set_markup?: &model.data.explorer.dir().name().map(|name| format!("<b>{name}</b>")),
                    },
                },
                pack_end = &gtk::Button {
                    set_icon_name: "accessories-text-editor-symbolic",
                    connect_clicked[sender] => move |_| {
                        sender.input(AppMsg::ShowLog);
                    },
                },
                pack_end = &gtk::Button {
                    set_icon_name: "view-refresh",
                    connect_clicked[sender] => move |_| {
                        sender.input(AppMsg::Refresh);
                    }
                }
            },

            gtk::Paned {
                    set_shrink_start_child: false,
                    set_shrink_end_child: false,
                    set_start_child = &gtk::ScrolledWindow {
                        set_hscrollbar_policy: gtk::PolicyType::Never,
                        set_vexpand: true,
                        set_margin_all: 5,
                        // Rules
                        set_child = Some(&gtk::ListBox) {
                            set_hexpand: true,
                            set_vexpand: true,
                            set_selection_mode: gtk::SelectionMode::Multiple,
                            set_activate_on_single_click: false,
                            add_css_class: "boxed-list",
                            // As a simple solution we just remove and reconstruct all the children
                            // every time something in the model changes. This should not be an issue
                            // because usually there is no more then a few rules for each directory.
                            // This might get replaced with Relm4 factories in future.
                            #[watch]
                            remove_all: (),
                            #[watch]
                            #[iterate]
                            append: model
                                .data
                                .db
                                .rules()
                                .get(model.data.explorer.dir().path())
                                .unwrap_or(&Vec::new())
                                .iter()
                                .enumerate()
                                .map(|(index, rule)| rule_view(index, rule))
                                .collect::<Vec<_>>()
                                .iter(),
                            #[watch]
                            append: &add_rule_button(&sender.input),
                        }
                    },
                    set_end_child = &gtk::ScrolledWindow {
                        set_hscrollbar_policy: gtk::PolicyType::Never,
                        set_vexpand: true,
                        // Files
                        set_child = Some(&gtk::GridView) {
                            set_vexpand: true,
                            set_enable_rubberband: true,
                            #[watch]
                            set_model: Some(&selection_model(model.data.explorer.items(), sender)),
                            set_factory: Some(&factory_identity()),
                            connect_activate[sender] => move |_, index| {
                                sender.input(AppMsg::OpenAt(index as usize))
                            }
                        }
                    }
                }
        }
    }

    fn post_view() {
        if !model.is_active {
            window.destroy();
        }
    }

    // Initialize the UI.
    fn init(
        db: Self::InitParams,
        root: &Self::Root,
        sender: &ComponentSender<App>,
    ) -> ComponentParts<Self> {
        let data = AppData::new(db);
        let mut model = App {
            executor: Executor::new(data.db.log()),
            data,
            root: root.clone(),
            is_active: true,
        };

        let widgets = view_output!();

        // TODO: Use actions to send messages to components

        SENDER.init(&sender.input);
        model.executor.restart(model.data.db.rules());

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: &ComponentSender<App>) {
        let App {
            data,
            executor,
            root,
            is_active,
        } = self;

        match message {
            AppMsg::Error(desc, error_text) => {
                ErrorDialog::builder()
                    .transient_for(root)
                    .launch((desc, error_text));
            }
            AppMsg::OpenAt(index) => {
                let item = data.explorer.items().get(index).cloned().unwrap();
                if item.tp() == &ItemType::Dir {
                    let path = item.path();
                    data.explorer
                        .open(path)
                        .or_show_error(&format!("Cannot open {:?}", path), sender);
                } else if item.tp() == &ItemType::File {
                    let path = item.path();
                    open::that(path).unwrap_or_else(|_| panic!("Can't open file at path {path:?}"));
                }
            }
            AppMsg::GoBack => data
                .explorer
                .go_back()
                .or_show_error("Cannot go back", sender),
            AppMsg::GoForward => data
                .explorer
                .go_forward()
                .or_show_error("Cannot go forward", sender),
            AppMsg::Refresh => data
                .explorer
                .refresh()
                .or_show_error("Cannot refresh", sender),
            AppMsg::Quit => {
                data.db.save();
                *is_active = false;
            }
            AppMsg::NewRuleRequest => {
                let rule = Rule::default();
                EditRuleWindow::builder()
                    .transient_for(root)
                    .launch((rule, EditMode::Create))
                    .forward(&sender.input, move |output| match output {
                        EditRuleOutput::Save(rule) => AppMsg::NewRule(rule),
                        _ => AppMsg::Ignore,
                    });
            }
            AppMsg::EditRuleRequest(index) => {
                let rule = data
                    .current_dir_rules()
                    .unwrap()
                    .get(index)
                    .unwrap()
                    .clone();
                EditRuleWindow::builder()
                    .transient_for(root)
                    .launch((rule, EditMode::Edit))
                    .forward(&sender.input, move |output| match output {
                        EditRuleOutput::Save(rule) => AppMsg::EditRule(index, rule),
                        EditRuleOutput::Cancel => AppMsg::Ignore,
                        EditRuleOutput::Delete => AppMsg::DeleteRule(index),
                    });
            }
            AppMsg::NewRule(rule) => {
                data.db
                    .rules_mut()
                    .entry(data.explorer.dir().path().to_owned())
                    .or_insert(vec![])
                    .push(rule);
                executor.restart(data.db.rules());
            }
            AppMsg::EditRule(index, rule) => {
                *data
                    .current_dir_rules_mut()
                    .unwrap()
                    .get_mut(index)
                    .unwrap() = rule;
                executor.restart(data.db.rules());
            }
            AppMsg::DeleteRule(index) => {
                data.current_dir_rules_mut().unwrap().remove(index);
                executor.restart(data.db.rules());
            }
            AppMsg::ShowLog => {
                LogWindow::builder()
                    .transient_for(root)
                    .launch(data.db.log().clone());
            }
            AppMsg::Ignore => {}
        }
    }
}

fn main() {
    let app: RelmApp<App> = RelmApp::new("cofee-on-the-desk.app.course_oop");
    relm4::set_global_css_from_file("assets/style.css");
    app.run(Database::load());
}

/// A selection model for the file view.
fn selection_model(items: &[Item], sender: &ComponentSender<App>) -> gtk::MultiSelection {
    let list_model = gtk::gio::ListStore::new(gtk::Box::static_type());
    for item in items {
        let tags = all_tags();
        let item_cloned = item.clone();
        view! {
            gtk_box = gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 10,

                gtk::Image {
                    set_from_file: Some(item.tp().icon_path()),
                    set_icon_size: gtk::IconSize::Large,
                },
                gtk::Label {
                    set_width_chars: 20,
                    set_ellipsize: gtk::pango::EllipsizeMode::Middle,
                    set_label?: &item.name(),
                },
                set_has_tooltip: true,
                connect_query_tooltip[sender] => move |_gtk_box, _x, _y, _keyboard, tooltip| -> bool {
                    let tag_labels = tags
                        .iter()
                        .filter(|tag| matches!(tag.is(&item_cloned), Ok(true)))
                        .map(tag_view)
                        .collect::<Vec<_>>();

                    view! {
                        tags = gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 10,
                            #[iterate]
                            append: &tag_labels,
                        }
                    }
                    tooltip.set_custom(Some(&tags));
                    true
                },
            }
        }
        list_model.append(&gtk_box);
    }
    let selection_model = gtk::MultiSelection::new(Some(&list_model));
    selection_model
}

/// A factory that produces an exact copy of its input.
fn factory_identity() -> gtk::SignalListItemFactory {
    let factory = gtk::SignalListItemFactory::new();
    factory.connect_bind(|_factory, list_item| {
        if list_item.child().is_none() {
            let widget = list_item
                .item()
                .and_then(|item| item.downcast::<gtk::Widget>().ok());
            list_item.set_child(widget.as_ref());
        }
    });
    factory
}

pub fn var_view(var: &Var) -> impl IsA<gtk::Widget> {
    let bin = adw::Bin::new();
    match var {
        Var::String { label, css_class } => bin.set_child(Some(
            &gtk::Label::builder()
                .label(label)
                .css_classes(css_class.map_or_else(Vec::new, |class| vec![class.into()]))
                .build(),
        )),
        Var::Tag(tag) => bin.set_child(Some(&tag_view(tag))),
        Var::Path(path) => bin.set_child(Some(
            &gtk::Button::builder()
                .label(&path.to_string_lossy())
                .css_classes(vec!["link".into()])
                .build(),
        )),
    }
    bin
}

/// Create a single tag widget.
pub fn tag_view(tag: &Tag) -> impl IsA<gtk::Widget> {
    view! {
        label = gtk::Label {
            set_margin_top: 2,
            set_margin_bottom: 2,
            set_label: tag.name(),
            set_tooltip_text: Some(tag.desc()),
            add_css_class: "tag",
        }
    }
    label
}

pub fn event_view(index: usize, event: &Event) -> impl IsA<gtk::Widget> {
    let vars = event.vars().iter().map(var_view).collect::<Vec<_>>();
    view! {
        container = gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_margin_all: 5,
            set_spacing: 15,
            gtk::Button {
                set_sensitive: false,
                set_margin_start: 5,
                gtk::Label { set_markup: &format!("<b>{}</b>", index + 1) },
                set_css_classes: &["circular", "dark-bg"]
            },
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 10,
                gtk::Image { set_icon_name: Some(event.gtk_icon()), },
                #[iterate]
                append: vars.iter(),
            }
        }
    }
    container
}

/// Create a single row that describes a rule.
pub fn rule_view(index: usize, rule: &Rule) -> impl IsA<gtk::Widget> {
    let row = adw::ExpanderRow::builder()
        .margin_start(5)
        .margin_end(5)
        .icon_name("starred-symbolic")
        .title(rule.title())
        .build();

    view! {
        edit_button = gtk::Button {
            set_margin_top: 10,
            set_margin_bottom: 10,
            set_css_classes: &["flat", "circular"],
            set_icon_name: "document-edit-symbolic",
            connect_clicked: move |_| {
                SENDER.send(AppMsg::EditRuleRequest(index));
            }
        }
    }

    row.add_action(&edit_button);

    for (index, event) in rule.events().iter().enumerate() {
        row.add_row(&event_view(index, event));
    }

    row
}

fn add_rule_button(sender: &relm4::Sender<AppMsg>) -> gtk::Button {
    view! {
            button = gtk::Button {
            set_icon_name: "list-add-symbolic",
            set_hexpand: true,
            connect_clicked[sender] => move |_| {
                sender.send(AppMsg::NewRuleRequest)
            }
        }
    }
    button
}
