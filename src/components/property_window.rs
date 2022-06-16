//! A window that shows `Item` properties.
use relm4::{
    gtk::{
        self,
        prelude::{BoxExt, GtkWindowExt, OrientableExt, WidgetExt},
    },
    view, ComponentParts, ComponentSender, SimpleComponent, WidgetPlus,
};

use crate::all_tags;
use crate::lib::Item;

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
            set_default_height: 450,
            set_title: Some(&format!("Properties of {}", item.path().to_string_lossy())),
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 15,
                set_spacing: 10,
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,
                    gtk::Image {
                        set_icon_name: Some("folder"),
                        set_icon_size: gtk::IconSize::Large,
                    },
                    gtk::Label { set_label?: &item.name() }
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
                            .unwrap_or_else(|_| String::from("(unknown)"))
                    }
                },
                gtk::Separator {},
                gtk::Label { set_label: "Tags" },
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 5,
                    gtk::FlowBox {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_hexpand: true,
                        set_selection_mode: gtk::SelectionMode::None,
                        #[iterate]
                        insert[-1]:
                            all_tags()
                            .into_iter()
                            .filter(|tag| if let Ok(b) = tag.is(&mut item) { b } else { false }).map(|tag| {
                                view! {
                                label = gtk::Label {
                                        set_height_request: 30,
                                        set_margin_top: 2,
                                        set_margin_bottom: 2,
                                        set_label: tag.name(),
                                        set_tooltip_text: Some(tag.desc()),
                                        add_css_class: "tag",
                                    }
                                }
                                label
                            })
                            .collect::<Vec<_>>()
                            .iter(),
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
