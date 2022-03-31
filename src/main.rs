mod lib;
use lib::{Item, ItemType, Tag};

mod db;
use db::Database;

mod fs;
use fs::Explorer;

mod components;
use components::error_dialog::ErrorDialog;

mod utils;
use utils::Expect;

use relm4::gtk::glib::FromVariant;
use relm4::gtk::prelude::{BoxExt, Cast, StaticType, StaticVariantType, ToVariant};
use relm4::{
    component, gtk, view, Component, ComponentParts, ComponentSender, RelmApp, Sender,
    SimpleComponent, WidgetPlus,
};
use serde::{Deserialize, Serialize};

use gtk::prelude::{ButtonExt, GtkWindowExt, OrientableExt, WidgetExt};
use std::cmp::Ordering;

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

pub struct AppComponents {}

impl AppComponents {
    pub fn new(
        _data: &AppData,
        _window: &gtk::ApplicationWindow,
        _sender: &Sender<AppMsg>,
    ) -> Self {
        AppComponents {}
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
    pub components: AppComponents,
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
                    set_sensitive: watch!(model.data.explorer.history().can_go_back()),
                    connect_clicked(sender) => move |_| {
                        sender.input(AppMsg::GoBack);
                    }
                },
                pack_start = &gtk::Button::from_icon_name("go-next") {
                    set_sensitive: watch!(model.data.explorer.history().can_go_forward()),
                    connect_clicked(sender) => move |_| {
                        sender.input(AppMsg::GoForward);
                    }
                },
                pack_end = &gtk::Button::from_icon_name("view-refresh") {
                    connect_clicked(sender) => move |_| {
                        sender.input(AppMsg::Refresh);
                    }
                }
            },
            &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                &gtk::CenterBox {
                    set_margin_all: 10,
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,
                    set_start_widget = Some(&gtk::Box) {
                        set_spacing: 10,
                        set_orientation: gtk::Orientation::Horizontal,

                        &gtk::Image {
                            set_from_file: Some(ItemType::Dir.icon_path()),
                            set_icon_size: gtk::IconSize::Large,
                        },
                        &gtk::Label { set_label?: watch!(&model.data.explorer.dir().name()) },
                    },
                    set_end_widget = Some(&gtk::Button) {
                        set_icon_name: "document-open-symbolic",
                        connect_clicked(sender) => move |_| {
                            //send!(input, AppMsg::Cmd(Command::ShowLogs));
                        },
                    }
                },
                &gtk::Paned {
                    set_shrink_start_child: false,
                    set_shrink_end_child: false,
                    set_start_child = &gtk::ScrolledWindow {
                        set_vexpand: true,
                        // Rules
                        set_child = Some(&gtk::ListBox) {
                            set_hexpand: true,
                            set_vexpand: true,
                            set_selection_mode: gtk::SelectionMode::Multiple,
                            set_activate_on_single_click: false,
                        }
                    },
                    set_end_child = &gtk::ScrolledWindow {
                        set_hscrollbar_policy: gtk::PolicyType::Never,
                        set_vexpand: true,
                        // Files
                        set_child = Some(&gtk::GridView) {
                            set_vexpand: true,
                            set_enable_rubberband: true,
                            set_model: watch!(Some(&selection_model(model.data.explorer.items(), model.data.db.tags()))),
                            set_factory: Some(&factory_identity()),
                            connect_activate(sender) => move |_, index| {
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
    fn init(
        db: Self::InitParams,
        root: &Self::Root,
        sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let data = AppData::new(db);
        let components = AppComponents::new(&data, root, &sender.input);

        let model = App {
            data,
            root: root.clone(),
            components,
            is_active: true,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: &ComponentSender<Self>) {
        let App {
            data,
            root,
            components,
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
fn selection_model(items: &[Item], tags: &[Tag]) -> gtk::MultiSelection {
    let list_model = gtk::gio::ListStore::new(gtk::Box::static_type());
    let mut items = items.to_vec();
    // Order items by name, folders first
    items.sort_by(|a, b| match (a.tp(), b.tp()) {
        (ItemType::Dir, ItemType::Dir) => a.name().cmp(&b.name()),
        (ItemType::Dir, _) => Ordering::Less,
        (_, ItemType::Dir) => Ordering::Greater,
        _ => a.name().cmp(&b.name()),
    });
    for item in items {
        let tags = tags.to_vec();
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
                connect_query_tooltip => move |_gtk_box, _x, _y, _keyboard, tooltip| -> bool {
                    let tag_labels = tags
                        .iter()
                        .filter(|category| matches!(category.is(&item), Ok(true)))
                        .map(|category| category.to_label())
                        .collect::<Vec<_>>();

                    view! {
                        tags = gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 10,
                            append: iterate!(&tag_labels)
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
