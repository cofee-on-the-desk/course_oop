use relm4::gtk::glib::FromVariant;
use relm4::gtk::prelude::{StaticVariantType, ToVariant};
use relm4::{
    component, gtk, ComponentParts, ComponentSender, RelmApp, Sender, SimpleComponent, WidgetPlus,
};
use serde::{Deserialize, Serialize};

use gtk::prelude::{ButtonExt, GtkWindowExt, OrientableExt, WidgetExt};

mod lib;

mod db;
use db::Database;

mod components;

mod utils;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AppMsg {
    // todo...
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
    pub db: Database,
}

impl AppData {
    pub fn new(db: Database) -> Self {
        AppData { db }
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
                    connect_clicked(sender) => move |_| {
                        //let _ = input.send(AppMsg::Fs(FsMsg::GoBack));
                    }
                },
                pack_start = &gtk::Button::from_icon_name("go-next") {
                    connect_clicked(sender) => move |_| {
                        //let _ = input.send(AppMsg::Fs(FsMsg::GoForward));
                    }
                },
                pack_end = &gtk::Button::from_icon_name("view-refresh") {
                    connect_clicked(sender) => move |_| {
                        //let _ = input.send(AppMsg::Fs(FsMsg::ReloadView));
                    }
                }
            },
            &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                &gtk::CenterBox {
                    set_margin_all: 10,
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,
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
                        set_child = Some(&gtk::GridView) {
                            set_vexpand: true,
                            set_enable_rubberband: true,
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
            AppMsg::Quit => *is_active = false,
        }
    }
}

fn main() {
    let app: RelmApp<App> = RelmApp::new("cofee-on-the-desk.app.course_oop");
    //relm4::set_global_css_from_file("styles.css");
    app.run(Database::default());
}
