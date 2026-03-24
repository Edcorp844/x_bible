use adw::prelude::*;
use relm4::prelude::*;
use std::sync::{Arc, RwLock};

use crate::features::{
    bible::components::page::helpers::PageDisplayConfig,
    core::{
        display_configurations::Config::TextConfig,
        module_engine::sword_engine::SwordEngine,
        pages::study::{
            bible_page_component::biblepage_root::{BiblePageRoot, BiblePageRootOutput},
            bible_search::search_page::SearchPage,
        },
    },
};

pub struct StudyPage {
    is_sidebar_visible: bool,
    bible_page: Controller<BiblePageRoot>,
    search_page: Controller<SearchPage>,
    config: TextConfig,
    show_search: bool,
}

#[derive(Debug)]
pub enum StudyPageInput {
    UpdateTheme,
    ToggleSearch,
}

#[derive(Debug)]
pub enum StudyPageOutPut {
    ToggleSidebar,
}

#[relm4::component(pub)]
impl Component for StudyPage {
    type Init = (Arc<SwordEngine>, bool);
    type Input = StudyPageInput;
    type Output = StudyPageOutPut;
    type CommandOutput = ();

    view! {
        adw::NavigationPage {
            set_title: "Bible Study",
            #[wrap(Some)]
            set_child = &adw::NavigationView {
                push = &adw::NavigationPage {
                    set_title: "Bible Study",
                    #[wrap(Some)]
                    set_child = &adw::ToolbarView {
                        #[name="header"]
                       add_top_bar = &adw::HeaderBar {
                            #[watch]
                            set_css_classes: &[format!("preview-area-{}", (model.config.read().unwrap().theme())).as_str()],

                            pack_start = &gtk::ToggleButton {
                                set_icon_name: "sidebar-show-symbolic",
                                #[watch]
                                set_active: model.is_sidebar_visible,
                                connect_clicked[sender] => move |_| {
                                    let _ = sender.output(StudyPageOutPut::ToggleSidebar);
                                }
                            },

                            #[wrap(Some)]
                            set_title_widget = &gtk::Box{
                                set_tooltip_text: Some("Search"),
                                add_css_class: "linked",
                                set_halign: gtk::Align::Center,


                                gtk::Button {
                                    #[wrap(Some)]
                                    set_child = &gtk::Box {
                                        set_halign: gtk::Align::Center,
                                        set_valign: gtk::Align::Center,
                                        set_width_request: 280,
                                        set_spacing: 8,

                                        set_hexpand: true,

                                        gtk::Image {
                                            set_icon_name: Some("system-search-symbolic"),
                                        },

                                        #[name = "version_label"]
                                        gtk::Label {
                                            set_label: "Search",
                                            add_css_class: "dim-label",
                                        },
                                    },
                                    add_css_class: "search-button",

                                    connect_clicked => move |_| {
                                        sender.input(StudyPageInput::ToggleSearch);
                                    }
                                }
                            }
                        },

                        #[name = "main_split"]
                        #[wrap(Some)]
                        set_content = &gtk::Stack {
                            set_transition_type: gtk::StackTransitionType::Crossfade,

                            #[watch]
                            set_visible_child_name: if model.show_search { "search" } else { "bible" },

                            add_named: (model.bible_page.widget(), Some("bible")),
                            add_named: (model.search_page.widget(), Some("search")),
                        }
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (engine, is_sidebar_visible) = init;

        let bible_page = BiblePageRoot::builder().launch(engine.clone()).forward(
            sender.input_sender(),
            move |message| match message {
                BiblePageRootOutput::UpdateTheme => StudyPageInput::UpdateTheme,
            },
        );

        let search_page = SearchPage::builder().launch(engine.clone()).detach();

        let model = StudyPage {
            is_sidebar_visible,
            bible_page: bible_page,
            search_page: search_page,
            config: Arc::new(RwLock::new(PageDisplayConfig::new())),
            show_search: false,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            StudyPageInput::ToggleSearch => {
                self.show_search = !self.show_search;
                let child_name = if self.show_search { "search" } else { "bible" };
                widgets.main_split.set_visible_child_name(child_name);
            }
            StudyPageInput::UpdateTheme => {
                let theme = self.config.read().unwrap().theme();

                //remove old theme classes first
                let themes = ["Classic", "Modern", "Default", "Compact"];
                for t in themes {
                    let class = format!("preview-area-{}", t);
                    if widgets.header.has_css_class(&class) {
                        widgets.header.remove_css_class(&class);
                    }
                    if widgets.main_split.has_css_class(&class) {
                        widgets.main_split.remove_css_class(&class);
                    }
                }

                widgets
                    .header
                    .add_css_class(&format!("preview-area-{}", theme));
                widgets
                    .main_split
                    .add_css_class(&format!("preview-area-{}", theme));
            }
        }
    }
}
