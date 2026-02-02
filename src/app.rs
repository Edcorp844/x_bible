use adw::prelude::*;
use relm4::prelude::*;
use std::{any::Any, collections::HashMap, sync::Arc};

use crate::features::{
    bible::components::page::{
        self,
        model::{StudyInput, StudyPage},
    },
    core::{
        components::sidebar::{NavigationPage, SideBar, SidebarMessage},
        pages::library::library_page::{LibraryPage, LibraryPageCategory},
        sword_engine::SwordEngine,
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
                // This forces the "overlay" mode regardless of window size
                set_collapsed: true,

                // Sync UI visibility to Model
                #[watch]
                set_show_sidebar: model.is_sidebar_visible,

                // Detect when the sidebar is closed by clicking away or swiping
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
                set_content = &adw::ToolbarView {
                    add_top_bar = &adw::HeaderBar {
                        #[wrap(Some)]
                        set_title_widget = &adw::WindowTitle {
                            set_title: "XBible",
                        },

                        pack_start = &gtk::ToggleButton {
                            set_icon_name: "sidebar-show-symbolic",
                            // Keep the button toggle state in sync with the actual visibility
                            #[watch]
                            set_active: model.is_sidebar_visible,

                            connect_clicked[sender] => move |_| {
                                sender.input(AppInputMessage::ToggleSidebar);
                            }
                        }
                    },

                     #[wrap(Some)]
                    set_content = &adw::Bin {
                        #[watch]
                        set_child: model.pages_cache.get(&model.current_page_key).map(|c| c.widget()),
                    },
                }
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
                .launch((engine.clone().mgr, "KJV".to_string()))
                .detach(),
        );

        let mut pages_cache = HashMap::new();
        pages_cache.insert("bible".to_string(), bible_page);

        let model = AppModel {
            side_bar,
            engine,
            is_sidebar_visible: false,
            pages_cache: pages_cache,
            current_page_key: "bible".to_string(),
        };

        let sidebar_widget = model.side_bar.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
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
                        NavigationPage::Bible => {}
                        NavigationPage::Library(category) => {
                            let libarary_page = LibraryPage::builder()
                                .launch((LibraryPageCategory::from_label(category.as_str()), self.engine.clone()))
                                .detach();
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
