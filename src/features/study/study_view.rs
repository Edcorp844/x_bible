use adw::prelude::*;
use gettextrs::gettext;
use relm4::prelude::*;

#[derive(Debug)]
struct TabSlot {
    page: adw::TabPage,
    content: gtk::Box,
}

#[derive(Debug)]
pub(crate) struct StudyView {
    tab_counter: usize,
    tabs: Vec<TabSlot>,
}

#[derive(Debug)]
pub enum StudyViewInput {
    NewTab,
    OpenSearch,
    TabSelected(adw::TabPage),
    CloseTab(adw::TabPage),
}

#[derive(Debug)]
pub enum StudyViewOutput {
    ShowError(String),
    ToggleSidebar,
}

#[relm4::component(pub)]
impl relm4::Component for StudyView {
    type Init = ();
    type Input = StudyViewInput;
    type Output = StudyViewOutput;
    type CommandOutput = ();

    view! {
        adw::NavigationPage {
            set_title: &gettext("Study"),

            #[wrap(Some)]
            set_child = &adw::TabOverview {
                #[wrap(Some)]
                set_child = &adw::ToolbarView {
                    add_top_bar = &adw::HeaderBar {
                        pack_start = &gtk::Button {
                            set_icon_name: "sidebar-show-symbolic",
                            set_valign: gtk::Align::Center,
                            set_tooltip_text: Some(&gettext("Toggle Sidebar")),
                            connect_clicked[sender] => move |_| {
                                let _ = sender.output(StudyViewOutput::ToggleSidebar);
                            },
                        },

                        pack_end = &adw::TabButton {
                            set_view: Some(&tab_view),
                            set_valign: gtk::Align::Center,
                            set_action_name: Some("overview.open"),
                            set_tooltip_text: Some(&gettext("Open Tab Overview")),
                        },

                        pack_end = &gtk::Button {
                            set_icon_name: "tab-new-symbolic",
                            set_valign: gtk::Align::Center,
                            set_tooltip_text: Some(&gettext("New Tab")),
                            connect_clicked[sender] => move |_| {
                                sender.input(StudyViewInput::NewTab);
                            },
                        },

                        pack_end = &gtk::Button {
                            set_icon_name: "edit-find-symbolic",
                            set_valign: gtk::Align::Center,
                            set_tooltip_text: Some(&gettext("Open Search Tab")),
                            connect_clicked[sender] => move |_| {
                                sender.input(StudyViewInput::OpenSearch);
                            },
                        },
                    },

                    add_top_bar = &adw::TabBar {
                        set_autohide: false,
                        set_view: Some(&tab_view),
                    },

                    #[wrap(Some)]
                    #[name = "tab_view"]
                    set_content = &adw::TabView {
                        connect_selected_page_notify[sender] => move |view| {
                            if let Some(page) = view.selected_page() {
                                sender.input(StudyViewInput::TabSelected(page));
                            }
                        },

                        connect_close_page[sender] => move |_, page| {
                            sender.input(StudyViewInput::CloseTab(page.clone()));
                            glib::Propagation::Stop
                        },
                    },
                },

                set_view: Some(&tab_view),
            },
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: relm4::prelude::ComponentSender<Self>,
    ) -> relm4::prelude::ComponentParts<Self> {
        let mut model = Self {
            tab_counter: 0,
            tabs: Vec::new(),
        };

        let widgets = view_output!();

        // Seed an initial placeholder tab so the workspace isn't blank on launch
        model.append_placeholder_tab(&widgets.tab_view, &gettext("Study Notes"), false);

        relm4::ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        _sender: relm4::prelude::ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            StudyViewInput::NewTab => {
                self.tab_counter += 1;
                let title = format!("{} {}", gettext("Study Tab"), self.tab_counter);
                self.append_placeholder_tab(&widgets.tab_view, &title, true);
            }

            StudyViewInput::OpenSearch => {
                self.tab_counter += 1;
                let title = format!("{} {}", gettext("Search"), self.tab_counter);
                self.append_placeholder_tab(&widgets.tab_view, &title, true);
            }

            StudyViewInput::TabSelected(_page) => {}

            StudyViewInput::CloseTab(page) => {
                widgets.tab_view.close_page_finish(&page, true);
                self.tabs.retain(|slot| slot.page != page);
            }
        }
    }
}

impl StudyView {
    fn append_placeholder_tab(&mut self, tab_view: &adw::TabView, title: &str, select: bool) {
        let placeholder_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(10)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .hexpand(true)
            .vexpand(true)
            .margin_top(24)
            .margin_bottom(24)
            .margin_start(24)
            .margin_end(24)
            .build();

        let icon = gtk::Image::builder()
            .icon_name("accessories-dictionary-symbolic")
            .pixel_size(40)
            .build();
        icon.add_css_class("dim-label");

        let heading = gtk::Label::builder()
            .label(title)
            .halign(gtk::Align::Center)
            .build();
        heading.add_css_class("title-2");

        let subtitle = gtk::Label::builder()
            .label(&gettext(
                "Study workspace placeholder — content coming soon",
            ))
            .halign(gtk::Align::Center)
            .build();
        subtitle.add_css_class("dim-label");

        placeholder_box.append(&icon);
        placeholder_box.append(&heading);
        placeholder_box.append(&subtitle);

        let page = tab_view.append(&placeholder_box);
        page.set_title(title);

        if select {
            tab_view.set_selected_page(&page);
        }

        self.tabs.push(TabSlot {
            page,
            content: placeholder_box,
        });
    }
}
