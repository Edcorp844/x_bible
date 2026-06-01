use adw::prelude::*;
use relm4::{Component, ComponentParts, prelude::*};

#[derive(Debug)]
pub enum NavigationPage {
    Bible,
    AudioBible,
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
            set_title: "XBible",
            set_hexpand: false,

            #[wrap(Some)]
            set_child = &adw::ToolbarView{
                add_top_bar=&adw::HeaderBar {
                    set_show_title: true,
                     pack_start = &gtk::Button {
                        set_icon_name: "system-search-symbolic",
                        set_tooltip_text: Some("Search"),
                        add_css_class: "flat",
                    },

                    pack_end = &gtk::MenuButton {
                        set_icon_name: "view-more-horizontal-symbolic",
                        set_tooltip_text: Some("Main Menu"),
                        add_css_class: "flat",

                        #[wrap(Some)]
                        set_popover = &gtk::PopoverMenu::from_model(Some(&{
                            let menu = gtk::gio::Menu::new();

                            let appearence_section = gtk::gio::Menu::new();

                            let appearance_item = gtk::gio::MenuItem::new(None, None);
                            appearance_item.set_attribute_value("custom", Some(&"theme_selector".to_variant()));
                            appearence_section.append_item(&appearance_item);
                            menu.append_section(None, &appearence_section);

                            let window_section = gtk::gio::Menu::new();
                            let new_window_item = gtk::gio::MenuItem::new(Some("New Window"), Some("app.new_window"));
                            new_window_item.set_attribute_value("accel", Some(&"<Primary>N".to_variant()));
                            window_section.append_item(&new_window_item);
                            menu.append_section(None, &window_section);

                            let section = gtk::gio::Menu::new();

                            let prefs_item = gtk::gio::MenuItem::new(Some("Preferences"), Some("app.preferences"));
                            prefs_item.set_attribute_value("accel", Some(&"<Primary>comma".to_variant()));
                            section.append_item(&prefs_item);

                            let shortcuts_item = gtk::gio::MenuItem::new(Some("Keyboard Shortcuts"), Some("app.shortcuts"));
                            shortcuts_item.set_attribute_value("accel", Some(&"<Primary>question".to_variant()));
                            section.append_item(&shortcuts_item);

                            let help_item = gtk::gio::MenuItem::new(Some("Help"), Some("app.help"));
                            help_item.set_attribute_value("accel", Some(&"F1".to_variant()));
                            section.append_item(&help_item);
                            section.append(Some("About xBible"), Some("app.about"));

                            menu.append_section(None, &section);

                            let quit_window_section = gtk::gio::Menu::new();
                            let quit_window_item = gtk::gio::MenuItem::new(Some("Quit"), Some("app.quit"));
                            quit_window_item.set_attribute_value("accel", Some(&"<Primary>Q".to_variant()));
                            quit_window_section.append_item(&quit_window_item);
                            menu.append_section(None, &quit_window_section);

                            menu
                        })) {
                            // 4. THE ACTUAL CIRCLES: Add the widget and link it to the "appearance" ID
                            add_child:(&{
                                let container = gtk::Box::builder()
                                    .orientation(gtk::Orientation::Horizontal)
                                    .halign(gtk::Align::Center)
                                    .spacing(12)
                                    .build();

                                container.add_css_class("themeselector");

                                let create_theme_btn = |style_class: &str| {
                                    let btn = gtk::CheckButton::builder().build();
                                    btn.add_css_class("theme-selector");
                                    btn.add_css_class(style_class);
                                    btn
                                };

                                let follow = create_theme_btn("follow");
                                let light = create_theme_btn("light");
                                let dark = create_theme_btn("dark");

                                follow.set_group(Some(&light));
                                dark.set_group(Some(&light));

                                container.append(&follow);
                                container.append(&light);
                                container.append(&dark);

                                container
                            }, "theme_selector")
                        }
                    },
                 },

                #[wrap(Some)]
                set_content = &gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,

                     gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 8,



                         gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 8,
                            set_margin_top: 20,

                            #[name = "pages"]
                            gtk::ListBox {
                                    // Start with None to prevent auto-selection during population
                                    set_selection_mode: gtk::SelectionMode::None,
                                    set_margin_horizontal: 12,
                                    add_css_class: "navigation-sidebar"
                            }
                        },

                        #[name = "tools_header"]
                        gtk::Box {
                            add_css_class: "sidebar-header-box",
                            set_margin_horizontal: 20,
                            gtk::Label {
                                set_label: "Tools",
                                add_css_class: "sidebar-section-title",
                                add_css_class: "dimmed",
                            },
                            gtk::Separator { set_hexpand: true, add_css_class: "spacer" },
                            #[name = "tools_chevron"]
                            gtk::Image { set_icon_name: Some("pan-down-symbolic"), add_css_class: "dimmed" }
                        },

                        #[name = "tools_revealer"]
                        gtk::Revealer {
                            set_reveal_child: true,
                            #[name = "tools_listbox"]
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
        let items = [
            ("bible-read-symbolic", "Study"),
            ("audio-input-microphone-symbolic", "Audio Bible"),
            ("my-store-symbolic", "Store"),
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

        let library = widgets.library.clone();

        let sender_clone = sender.clone();
        listbox.connect_row_activated(move |_, row| {
            library.unselect_all();

            if row.widget_name().as_str() == "Study" {
                let _ = sender_clone
                    .output_sender()
                    .send(SidebarMessage::SelectPage(NavigationPage::Bible));
            }
            if row.widget_name().as_str() == "Audio Bible" {
                let _ = sender_clone
                    .output_sender()
                    .send(SidebarMessage::SelectPage(NavigationPage::AudioBible));
            }
            if row.widget_name().as_str() == "Store" {
                let _ = sender_clone
                    .output_sender()
                    .send(SidebarMessage::SelectPage(NavigationPage::Store));
            }

            let _ = sender_clone
                .output_sender()
                .send(SidebarMessage::ToggleSidebar);
        });
    }

    fn render_library_list(widgets: &SideBarWidgets, sender: &ComponentSender<Self>) {
        let listbox = &widgets.library;
        let items = [
            ("bibles-symbolic", "Bible Versions"),
            ("commentaries-symbolic", "Commentaries"),
            ("dictionaries", "Dictionaries"),
            ("audio-bible-symbolic", "Audio Bibles"),
            ("map-symbolic", "Maps"),
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
            let _ = sender_clone
                .output_sender()
                .send(SidebarMessage::ToggleSidebar);
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
