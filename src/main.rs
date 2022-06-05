mod lib;
use lib::{Event, Item, ItemType, Rule, Tag, Var};

mod db;
use db::Database;

mod fs;
use fs::Explorer;

mod components;
use components::error_dialog::ErrorDialog;

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

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AppMsg {
    Error(String, String),
    GoBack,
    GoForward,
    OpenAt(usize),
    Refresh,
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
}

pub struct App {
    pub data: AppData,
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
                pack_end = &gtk::Button::from_icon_name("view-refresh") {
                    connect_clicked[sender] => move |_| {
                        sender.input(AppMsg::Refresh);
                    }
                }
            },
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                gtk::CenterBox {
                    set_margin_all: 10,
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,
                    set_start_widget = Some(&gtk::Box) {
                        set_spacing: 10,
                        set_orientation: gtk::Orientation::Horizontal,

                        gtk::Image {
                            set_from_file: Some(ItemType::Dir.icon_path()),
                            set_icon_size: gtk::IconSize::Large,
                        },
                        gtk::Label {
                            #[watch]
                            set_label?: &model.data.explorer.dir().name()
                        },
                    },
                    set_end_widget = Some(&gtk::Button) {
                        set_icon_name: "document-open-symbolic",
                        connect_clicked[sender] => move |_| {
                            //send!(input, AppMsg::Cmd(Command::ShowLogs));
                        },
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
                                .map(|rule| rule_view(rule, sender))
                                .collect::<Vec<_>>()
                                .iter()
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
                            set_model: Some(&selection_model(model.data.explorer.items(), model.data.db.tags(), &sender)),
                            set_factory: Some(&factory_identity()),
                            connect_activate[sender] => move |_, index| {
                                sender.input(AppMsg::OpenAt(index as usize))
                            }
                        }
                    }
                },
            }
        }
    }

    fn post_view() {
        if !model.is_active {
            window.destroy();
        }
    }

    // Initialize the UI.
    fn init(db: Self::InitParams, root: &Self::Root, sender: &AppSender) -> ComponentParts<Self> {
        let data = AppData::new(db);
        let model = App {
            data,
            root: root.clone(),
            is_active: true,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: &AppSender) {
        let App {
            data,
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
            AppMsg::Quit => *is_active = false,
        }
    }
}

fn main() {
    let app: RelmApp<App> = RelmApp::new("cofee-on-the-desk.app.course_oop");
    relm4::set_global_css_from_file("assets/style.css");
    app.run(Database::default());
}

/// A selection model for the file view.
fn selection_model(items: &[Item], tags: &[Tag], sender: &AppSender) -> gtk::MultiSelection {
    let list_model = gtk::gio::ListStore::new(gtk::Box::static_type());
    for item in items {
        let tags = tags.to_vec();
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
                        .map(|tag| tag_view(tag, &sender))
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

pub fn var_view(var: &Var, sender: &AppSender) -> impl IsA<gtk::Widget> {
    let bin = adw::Bin::new();
    match var {
        Var::String(s) => bin.set_child(Some(&gtk::Label::new(Some(s)))),
        Var::Tag(tag) => bin.set_child(Some(&tag_view(tag, sender))),
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
pub fn tag_view(tag: &Tag, sender: &AppSender) -> impl IsA<gtk::Widget> {
    view! {
        label = gtk::Label {
            set_label: &tag.emoji_name(),
            set_tooltip_text?: Some(tag.desc()),
        }
    }
    label
}

pub fn event_view(event: &Event, sender: &AppSender) -> impl IsA<gtk::Widget> {
    let vars = event
        .vars()
        .iter()
        .map(|var| var_view(var, sender))
        .collect::<Vec<_>>();
    view! {
        container = gtk::Box {
            set_margin_all: 5,
            set_spacing: 5,
            gtk::Image { set_icon_name: Some(event.gtk_icon()) },
            #[iterate]
            append: vars.iter(),
        }
    }
    container
}

/// Create a single row that describes a rule.
pub fn rule_view(rule: &Rule, sender: &AppSender) -> impl IsA<gtk::Widget> {
    let row = adw::ExpanderRow::builder()
        .margin_top(5)
        .margin_bottom(5)
        .margin_start(5)
        .margin_end(5)
        .icon_name("starred-symbolic")
        .title("Test Rule")
        .build();

    for event in rule.events() {
        row.add_row(&event_view(event, sender));
    }

    row
}

type AppSender = ComponentSender<App>;
