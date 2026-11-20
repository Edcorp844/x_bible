use std::collections::HashMap;

use adw::prelude::*;
use relm4::{Component, Controller};

use crate::{
    core::engine::SharedEngine,
    features::{
        store::store_view::StoreView,
        study::study_view::{StudyView, StudyViewOutput},
    },
    navigation::{page_controller::PageController, xbible_navigationpage::XBibleNavigationPage},
    workers::store_worker::StoreWorker,
};

#[derive(Debug)]
pub struct XBibleApp {
    sidebar_collapsed: bool,
    engine: SharedEngine,
    pages_cache: HashMap<String, PageController>,
    current_page_key: String,
}

#[derive(Debug)]
pub enum XBibleAppInput {
    SelectPage(XBibleNavigationPage),
    ShowError(String),
    ToggleSidebar,
}

#[derive(Debug)]
pub enum XBibleAppOutput {}

#[relm4::component(pub)]
impl Component for XBibleApp {
    type Init = SharedEngine;
    type Input = XBibleAppInput;
    type Output = XBibleAppOutput;
    type CommandOutput = ();

    view! {
        adw::Window {
            set_title: Some("XBible"),
            set_default_size: (1080, 800),
            #[wrap(Some)]
            set_content = &adw::OverlaySplitView {
                set_sidebar_position: gtk::PackType::Start,
                #[watch]
                set_collapsed: model.sidebar_collapsed,


                #[wrap(Some)]
                set_sidebar = &adw::ToolbarView {
                    add_top_bar = &adw::HeaderBar {},

                    #[wrap(Some)]
                    set_content = &adw::Sidebar {
                        append = adw::SidebarSection {
                            append = adw::SidebarItem {
                                set_icon_name: Some(&XBibleNavigationPage::Study.icon_name()),
                                set_title: Some(&XBibleNavigationPage::Study.title()),
                            },
                            append = adw::SidebarItem {
                                set_icon_name: Some(&XBibleNavigationPage::Store.icon_name()),
                                set_title: Some(&XBibleNavigationPage::Store.title()),
                            }
                        },

                        connect_activated[sender] => move |bar, _index| {
                            if let Some(title) = bar.selected_item().and_then(|i| i.title()) {
                                sender.input(XBibleAppInput::SelectPage(
                                    XBibleNavigationPage::from_name(title.as_str()),
                                ));
                            }
                        }
                    }
                },

                #[wrap(Some)]
                #[name = "toast_overlay"]
                set_content = &adw::ToastOverlay {
                    #[name="main_content"]
                    adw::NavigationPage {
                        set_title: "XBible",
                        set_child = model.pages_cache.get(&model.current_page_key).map(PageController::widget),
                    }
                }
            }
        }
    }

    fn init(
        engine: Self::Init,
        root: Self::Root,
        sender: relm4::prelude::ComponentSender<Self>,
    ) -> relm4::prelude::ComponentParts<Self> {
        let initial_page =
            StudyView::builder()
                .launch(())
                .forward(sender.input_sender(), move |message| match message {
                    StudyViewOutput::ShowError(error) => XBibleAppInput::ShowError(error),
                    StudyViewOutput::ToggleSidebar => XBibleAppInput::ToggleSidebar,
                });
        let mut pages_cache = HashMap::new();
        pages_cache.insert(
            XBibleNavigationPage::Study.to_key(),
            PageController::Study(initial_page),
        );

        let model = Self {
            sidebar_collapsed: false,
            engine,
            pages_cache,
            current_page_key: XBibleNavigationPage::Study.to_key(),
        };

        let widgets = view_output!();
        relm4::ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: relm4::prelude::ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            XBibleAppInput::SelectPage(page) => {
                let key = page.to_key();

                if !self.pages_cache.contains_key(&key) {
                    let navigation_page = match page {
                        XBibleNavigationPage::Study => {
                            PageController::Study(StudyView::builder().launch(()).forward(
                                sender.input_sender(),
                                move |message| match message {
                                    StudyViewOutput::ShowError(error) => {
                                        XBibleAppInput::ShowError(error)
                                    }
                                    StudyViewOutput::ToggleSidebar => XBibleAppInput::ToggleSidebar,
                                },
                            ))
                        }
                        XBibleNavigationPage::Store => PageController::Store(
                            StoreView::builder().launch(self.engine.clone()).detach(),
                        ),
                    };

                    self.pages_cache.insert(key.clone(), navigation_page);
                }
                self.current_page_key = key;

                let child = self
                    .pages_cache
                    .get(&self.current_page_key)
                    .map(|p| p.widget().upcast_ref::<gtk::Widget>());
                widgets.main_content.set_child(child);
            }
            XBibleAppInput::ShowError(error) => {
                error!("{error}");
            }
            XBibleAppInput::ToggleSidebar => self.sidebar_collapsed = !self.sidebar_collapsed,
        }
    }
}
