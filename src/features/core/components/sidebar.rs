use adw::prelude::*;
use relm4::{Component, ComponentParts, prelude::*};

#[derive(Debug)]
pub enum NavigationPage {
    Bible,
    Library(String),
    Store,
}

impl NavigationPage {
    pub fn to_key(&self) -> String {
        format!("{:?}", self)
    }
}

pub struct SideBar {}

#[derive(Debug)]
pub enum SidebarMessage {
    ToggleSidebar,
    SelectPage(NavigationPage),
}

#[relm4::component(pub)]
impl Component for SideBar {
    type Init = ();
    type Input = ();
    type Output = SidebarMessage;
    type CommandOutput = ();

    view! {
         adw::NavigationPage {
            set_title: "FrostNews",
            set_hexpand: true,

            #[wrap(Some)]
            set_child = &adw::ToolbarView{
                add_top_bar=&adw::HeaderBar {
                    set_show_title: false,
                    pack_end = &gtk::Button {
                        set_icon_name: "sidebar-show-symbolic",
                        set_tooltip_text: Some("Hide Sidebar"),
                        add_css_class: "flat",
                        connect_clicked[sender] => move |_| {
                            let _ = sender.output(SidebarMessage::ToggleSidebar);
                        }
                    }
                },

                #[wrap(Some)]
                set_content = &gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,

                     gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 8,

                        gtk::Label{
                            set_label: "XBible",
                            add_css_class: "title-1",
                            add_css_class: "accent",
                            add_css_class: "app-title",
                            set_margin_all: 20,
                            set_xalign: 0.0,
                            
                        },

                         gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 8,

                         #[name = "pages"]
                        gtk::ListBox {
                                // Start with None to prevent auto-selection during population
                                set_selection_mode: gtk::SelectionMode::None,
                                set_margin_horizontal: 12,
                                add_css_class: "navigation-sidebar"
                        }
                    },

                          #[name = "library_header"]
                        gtk::Box {
                            add_css_class: "sidebar-header-box",
                            set_margin_horizontal: 20,
                            gtk::Label {
                                set_label: "Library",
                                add_css_class: "sidebar-section-title",
                                add_css_class: "dimmed",
                            },
                            gtk::Separator { set_hexpand: true, add_css_class: "spacer" },
                            #[name = "library_chevron"]
                            gtk::Image { set_icon_name: Some("pan-down-symbolic"), add_css_class: "dimmed" }
                        },
                        #[name = "library_revealer"]
                        gtk::Revealer {
                            set_reveal_child: true,
                            #[name = "library"]
                            gtk::ListBox {
                                // Start with None to prevent auto-selection during population
                                set_selection_mode: gtk::SelectionMode::None,
                                set_margin_horizontal: 12,
                                add_css_class: "navigation-sidebar"
                            }
                        },

                     }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let model = SideBar {};

        let widgets = view_output!();

        Self::setup_collapsible_section(
            &widgets.library_header,
            &widgets.library_revealer,
            &widgets.library_chevron,
        );

        Self::render_pages_list(&widgets, &sender);
        Self::render_library_list(&widgets, &sender);

        widgets.library.set_can_focus(false);
        widgets.pages.set_can_focus(false);

        widgets
            .library
            .set_selection_mode(gtk::SelectionMode::Single);
        widgets.pages.set_selection_mode(gtk::SelectionMode::Single);

        widgets.library.unselect_all();
        widgets.pages.unselect_all();

        if let Some(row) = widgets
            .pages
            .first_child()
            .and_then(|w| w.dynamic_cast::<gtk::ListBoxRow>().ok())
        {
            widgets.pages.select_row(Some(&row));
        }

        ComponentParts { model, widgets }
    }
}

impl SideBar {
    fn render_pages_list(widgets: &SideBarWidgets, sender: &ComponentSender<Self>) {
        let listbox = &widgets.pages;
        let items = [("bibles-symbolic", "Bible"), ("store-symbolic", "store")];

        for (icon_name, label_text) in items {
            let row_box = gtk::Box::builder()
                .spacing(16)
                .css_classes(vec!["Category"])
                .build();

            let icon = gtk::Image::from_icon_name(icon_name);
            icon.set_pixel_size(22);
            icon.set_margin_start(8);
            icon.add_css_class("sidebar_icon");

            let label = gtk::Label::builder()
                .label(label_text)
                .css_classes(vec!["sidebar-label"])
                .build();

            row_box.append(&icon);
            row_box.append(&label);

            let row = gtk::ListBoxRow::builder()
                .name(label_text)
                .child(&row_box)
                .margin_end(0)
                .margin_start(0)
                .build();

            listbox.append(&row);
        }

        let library = widgets.library.clone();

        let sender_clone = sender.clone();
        listbox.connect_row_activated(move |_, row| {
            library.unselect_all();

            if row.widget_name().as_str() == "Bible" {
                let _ = sender_clone
                    .output_sender()
                    .send(SidebarMessage::SelectPage(NavigationPage::Bible));
            }
            if row.widget_name().as_str() == "Store" {
                let _ = sender_clone
                    .output_sender()
                    .send(SidebarMessage::SelectPage(NavigationPage::Store));
            }
        });
    }

    fn render_library_list(widgets: &SideBarWidgets, sender: &ComponentSender<Self>) {
        let listbox = &widgets.library;
        let items = [
            // Bibles
            ("bibles-symbolic", "Bible Versions"),
            // Commentaries (Library/Reference look)
            ("commentaries-symbolic", "Commentaries"),
            // Dictionaries (Alphabetical/Search look)
            ("dictionaries", "Dictionaries"),
            // Audio Bibles (Standard audio/speaker)
            ("audio-bible-symbolic", "Audio Bibles"),
            // Maps / Geography
            ("map-symbolic", "Maps"),
            // General Books / Glossaries
            ("books-symbolic", "General Books"),
        ];

        for (icon_name, label_text) in items {
            let row_box = gtk::Box::builder()
                .spacing(16)
                .css_classes(vec!["Category"])
                .build();

            let icon = gtk::Image::from_icon_name(icon_name);
            icon.set_pixel_size(22);
            icon.set_margin_start(8);
            icon.add_css_class("sidebar_icon");

            let label = gtk::Label::builder()
                .label(label_text)
                .css_classes(vec!["sidebar-label"])
                .build();

            row_box.append(&icon);
            row_box.append(&label);

            let row = gtk::ListBoxRow::builder()
                .name(label_text)
                .child(&row_box)
                .margin_end(0)
                .margin_start(0)
                .build();

            listbox.append(&row);
        }

        let pages = widgets.pages.clone();

        let sender_clone = sender.clone();
        listbox.connect_row_activated(move |_, row| {
            pages.unselect_all();

            let _ = sender_clone
                .output_sender()
                .send(SidebarMessage::SelectPage(NavigationPage::Library(
                    row.widget_name().as_str().to_string(),
                )));
        });
    }

    pub fn setup_collapsible_section(
        header: &gtk::Box,
        revealer: &gtk::Revealer,
        chevron: &gtk::Image,
    ) {
        let r = revealer.clone();
        let c = chevron.clone();
        let gesture = gtk::GestureClick::new();

        gesture.connect_released(move |_, _, _, _| {
            let is_revealing = !r.reveals_child();
            r.set_reveal_child(is_revealing);
            c.set_icon_name(Some(if is_revealing {
                "pan-down-symbolic"
            } else {
                "pan-end-symbolic"
            }));
        });
        header.add_controller(gesture);
    }
}
