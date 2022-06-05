//! A window that shows `Item` properties.
use relm4::{
    gtk::{
        self,
        prelude::{BoxExt, GtkWindowExt, OrientableExt, WidgetExt},
    },
    ComponentParts, ComponentSender, SimpleComponent, WidgetPlus,
};

use crate::lib::{Item, Tag};

pub struct PropertyWindow;

#[relm4::component(pub)]
impl SimpleComponent for PropertyWindow {
    type Widgets = PropertyWindowWidgets;

    type InitParams = Item;

    type Input = ();
    type Output = ();

    view! {
        gtk::Window {
            set_modal: true,
            set_default_width: 400,
            set_default_height: 600,
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 15,
                set_spacing: 10,
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,
                    /*gtk::Image {
                        set_from_file: Some(item.tp().icon_path()),
                        set_icon_size: gtk::IconSize::Large,
                    },*/
                    gtk::Label { set_label?: &item.name() }
                },
                gtk::Label {
                    set_label: "General",
                    set_halign: gtk::Align::Start,
                    inline_css: "opacity: 0.5",
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,
                    gtk::Label { set_label: "Location:" },
                    gtk::Label {
                        set_ellipsize: gtk::pango::EllipsizeMode::Middle,
                        set_label: &item.path().to_string_lossy(),
                    }
                },
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,
                    gtk::Label { set_label: "Size:" },
                    gtk::Label {
                        set_label: &item.size()
                            .map(|bytes| bytes.get_appropriate_unit(true).to_string())
                            .unwrap_or(String::from("(unknown)"))
                    }
                },
                gtk::Separator {},
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,
                    gtk::Label { set_label: "Tags:" },
                    gtk::FlowBox {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_hexpand: true,
                        set_selection_mode: gtk::SelectionMode::None,
                    }
                },
            }
        }
    }

    fn init(
        mut item: Self::InitParams,
        root: &Self::Root,
        _sender: &ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = PropertyWindow;
        let widgets = view_output!();
        root.present();
        ComponentParts { model, widgets }
    }
}
