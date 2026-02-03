use adw::prelude::*;
use relm4::prelude::*;
use std::{collections::HashMap, sync::Arc};

use crate::features::core::{
    components::sidebar::{NavigationPage, SideBar, SidebarMessage},
    module_engine::sword_engine::SwordEngine,
    pages::{
        library::library_page::{LibraryPage, LibraryPageCategory, LibraryPageOutput},
        study::study_page::{StudyPage, StudyPageOutPut},
    },
};

enum PageController {
    Bible(Controller<StudyPage>),
    Library(Controller<LibraryPage>),
}

impl PageController {
    fn widget(&self) -> &adw::NavigationPage {
        match self {
            Self::Bible(c) => c.widget(),
            Self::Library(c) => c.widget(),
        }
    }
}
pub struct AppModel {
    side_bar: Controller<SideBar>,
    pages_cache: HashMap<String, PageController>,
    engine: Arc<SwordEngine>,
    is_sidebar_visible: bool,
    current_page_key: String,
}

#[derive(Debug)]
pub enum AppInputMessage {
    ToggleSidebar,
    SetContentPage(NavigationPage),
    SetSidebarVisibility(bool),
}

#[relm4::component(pub)]
impl SimpleComponent for AppModel {
    type Init = ();
    type Input = AppInputMessage;
    type Output = ();

    view! {
        adw::ApplicationWindow {
            set_default_width: 1000,
            set_default_height: 800,

            #[name = "split_view"]
            adw::OverlaySplitView {
                set_collapsed: true,

                #[watch]
                set_show_sidebar: model.is_sidebar_visible,

                connect_show_sidebar_notify[sender] => move |view| {
                    sender.input(AppInputMessage::SetSidebarVisibility(view.shows_sidebar()));
                },

                #[wrap(Some)]
                set_sidebar = &gtk::Box {
                    add_css_class: "sidebar-view",
                    #[local_ref]
                    sidebar_widget -> adw::NavigationPage {},
                },

                #[wrap(Some)]
                set_content = &adw::Bin {
                    #[watch]
                    set_child: model.pages_cache.get(&model.current_page_key).map(|c| c.widget()),
                },
            }
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let engine = Arc::new(SwordEngine::new());

        let side_bar = SideBar::builder()
            .launch(())
            .forward(sender.input_sender(), |message| match message {
                SidebarMessage::ToggleSidebar => AppInputMessage::ToggleSidebar,
                SidebarMessage::SelectPage(page) => AppInputMessage::SetContentPage(page),
            });

        let bible_page = PageController::Bible(
            StudyPage::builder()
                .launch((engine.clone(), false))
                .forward(sender.input_sender(), |message| match message {
                    StudyPageOutPut::ToggleSidebar => AppInputMessage::ToggleSidebar,
                }),
        );

        let mut pages_cache = HashMap::new();
        pages_cache.insert(NavigationPage::Bible.to_key(), bible_page);

        let model = AppModel {
            side_bar,
            engine,
            is_sidebar_visible: false,
            pages_cache: pages_cache,
            current_page_key: NavigationPage::Bible.to_key(),
        };

        let sidebar_widget = model.side_bar.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            AppInputMessage::ToggleSidebar => {
                self.is_sidebar_visible = !self.is_sidebar_visible;
            }
            AppInputMessage::SetSidebarVisibility(visible) => {
                // Only update if the state actually changed to prevent loops
                if self.is_sidebar_visible != visible {
                    self.is_sidebar_visible = visible;
                }
            }
            AppInputMessage::SetContentPage(page) => {
                let key = page.to_key();

                if !self.pages_cache.contains_key(&key) {
                    match page {
                        NavigationPage::Bible => {
                            let bible_page = PageController::Bible(
                                StudyPage::builder()
                                    .launch((self.engine.clone(), false))
                                    .forward(sender.input_sender(), |message| match message {
                                        StudyPageOutPut::ToggleSidebar => {
                                            AppInputMessage::ToggleSidebar
                                        }
                                    }),
                            );
                            self.pages_cache.insert(key.clone(), bible_page);
                        }
                        NavigationPage::Library(category) => {
                            let libarary_page = LibraryPage::builder()
                                .launch((
                                    LibraryPageCategory::from_label(category.as_str()),
                                    self.engine.clone(),
                                    self.is_sidebar_visible,
                                ))
                                .forward(sender.input_sender(), |message| match message {
                                    LibraryPageOutput::ToggleSidebar => {
                                        AppInputMessage::ToggleSidebar
                                    }
                                });
                            self.pages_cache
                                .insert(key.clone(), PageController::Library(libarary_page));
                        }
                        _ => {}
                    }
                }

                self.current_page_key = key;
            }
        }
    }
}
