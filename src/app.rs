use ::xbible_engine::engines::{
    audio_engine::engine::AudioEngine, xbible_engine::engine::XBibleEngine,
};
use adw::prelude::*;
use relm4::prelude::*;
use std::fmt::Debug;
use std::{collections::HashMap, sync::Arc};

use crate::features::core::pages::audio_bible::audio_bible_page::AudioBibleOutput;
use crate::features::core::pages::audio_bible::persistent_control::AudioPersistentControl;
use crate::features::core::{
    components::sidebar::{NavigationPage, SideBar, SidebarMessage},
    pages::{
        audio_bible::audio_bible_page::AudioBiblePage,
        library::library_page::{LibraryPage, LibraryPageCategory, LibraryPageOutput},
        study::study_page::{StudyPage, StudyPageOutPut},
    },
};

enum PageController {
    Bible(Controller<StudyPage>),
    AudioBible(Controller<AudioBiblePage>),
    //Store(Controller<StudyPage>),
    Library(Controller<LibraryPage>),
}

impl PageController {
    fn widget(&self) -> &adw::NavigationPage {
        match self {
            Self::Bible(c) => c.widget(),
            Self::AudioBible(c) => c.widget(),
            // Self::Store(c) => c.widget(),
            Self::Library(c) => c.widget(),
        }
    }
}

pub struct AppModel {
    side_bar: Controller<SideBar>,
    pages_cache: HashMap<String, PageController>,

    engine: Option<Arc<XBibleEngine>>,
    audio_engine: Arc<AudioEngine>,
    is_engine_ready: bool,
    engine_error: Option<String>,

    is_sidebar_visible: bool,
    current_page_key: String,

    persistent_player: Controller<AudioPersistentControl>,
}

#[derive(Debug)]
pub enum AppInputMessage {
    ToggleSidebar,
    SetContentPage(NavigationPage),
    SetSidebarVisibility(bool),

    EngineInitializationSuccess(Arc<XBibleEngine>),
    EngineInitializationFailed(String),
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
                set_sidebar_width_fraction: 0.2,
                set_max_sidebar_width: 150.0,

                #[watch]
                set_show_sidebar: model.is_sidebar_visible,

                connect_show_sidebar_notify[sender] => move |view| {
                    sender.input(AppInputMessage::SetSidebarVisibility(view.shows_sidebar()));
                },

                #[wrap(Some)]
                set_sidebar = &gtk::Box {
                    #[local_ref]
                    sidebar_widget -> adw::NavigationPage {},
                },

                #[wrap(Some)]
                set_content = &adw::Bin {
                   #[name = "content_stack"]
                    gtk::Stack {
                        set_transition_type: gtk::StackTransitionType::Crossfade,

                        // Persistent Sub-View 1: Loading view state wrapper
                        add_named[Some("loading")] = &gtk::CenterBox {
                            set_center_widget: Some(&gtk::Spinner::builder().spinning(true)
                                .css_classes(vec![String::from("loading-spinner")])
                                .build()),
                        },

                        // Persistent Sub-View 2: Error view state container
                        add_named[Some("error")] = &gtk::CenterBox {
                            #[watch]
                            set_center_widget: model.engine_error.as_ref().map(|msg| {
                                gtk::Label::builder()
                                    .label(msg)
                                    .css_classes(vec![String::from("error-state-label")])
                                    .build()
                                    .upcast::<gtk::Widget>()
                            }).as_ref(),
                        },

                        // Persistent Sub-View 3: Workspace layout
                       add_named[Some("workspace")] = &adw::Bin {
                            gtk::Overlay {
                                #[watch]
                                set_child: model.pages_cache.get(&model.current_page_key).map(|c| c.widget()),
                                add_overlay = model.persistent_player.widget() {
                                    set_valign: gtk::Align::End,
                                    set_halign: gtk::Align::Center,
                                }
                            }
                        },

                        // Track state modifications to flip active layout index targets smoothly
                        #[watch]
                        set_visible_child_name: if model.engine_error.is_some() {
                            "error"
                        } else if !model.is_engine_ready {
                            "loading"
                        } else {
                            "workspace"
                        },
                    }
                },
            }
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let engine_sender = sender.clone();
        std::thread::spawn(
            move || match std::panic::catch_unwind(|| XBibleEngine::new()) {
                Ok(loaded_engine) => {
                    engine_sender
                        .input(AppInputMessage::EngineInitializationSuccess(loaded_engine));
                }
                Err(_) => {
                    engine_sender.input(AppInputMessage::EngineInitializationFailed(String::from(
                        "Failed to initialize SWORD background subsystems.",
                    )));
                }
            },
        );

        let side_bar = SideBar::builder()
            .launch(())
            .forward(sender.input_sender(), |message| match message {
                SidebarMessage::ToggleSidebar => AppInputMessage::ToggleSidebar,
                SidebarMessage::SelectPage(page) => AppInputMessage::SetContentPage(page),
            });

        let audio_engine = Arc::new(AudioEngine::new());

        let persistent_player = AudioPersistentControl::builder()
            .launch(audio_engine.clone()) // Pass engine reference down
            .detach();

        let model = AppModel {
            side_bar,
            pages_cache: HashMap::new(),
            engine: None,
            audio_engine,
            is_engine_ready: false,
            engine_error: None,
            is_sidebar_visible: false,
            current_page_key: NavigationPage::Bible.to_key(),
            persistent_player,
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
                if self.is_sidebar_visible != visible {
                    self.is_sidebar_visible = visible;
                }
            }

            AppInputMessage::EngineInitializationSuccess(engine_arc) => {
                self.engine = Some(engine_arc.clone());
                self.is_engine_ready = true;

                let bible_page = PageController::Bible(
                    StudyPage::builder()
                        .launch((engine_arc.clone(), self.is_sidebar_visible))
                        .forward(sender.input_sender(), |message| match message {
                            StudyPageOutPut::ToggleSidebar => AppInputMessage::ToggleSidebar,
                        }),
                );
                self.pages_cache
                    .insert(NavigationPage::Bible.to_key(), bible_page);
            }

            AppInputMessage::EngineInitializationFailed(err_string) => {
                self.engine_error = Some(err_string);
                self.is_engine_ready = false;
            }

            AppInputMessage::SetContentPage(page) => {
                let key = page.to_key();

                let Some(ref active_engine) = self.engine else {
                    return;
                };

                if !self.pages_cache.contains_key(&key) {
                    match page {
                        NavigationPage::Bible => {
                            let bible_page = PageController::Bible(
                                StudyPage::builder()
                                    .launch((active_engine.clone(), self.is_sidebar_visible))
                                    .forward(sender.input_sender(), |message| match message {
                                        StudyPageOutPut::ToggleSidebar => {
                                            AppInputMessage::ToggleSidebar
                                        }
                                    }),
                            );
                            self.pages_cache.insert(key.clone(), bible_page);
                        }
                        NavigationPage::AudioBible => {
                            let audio_page = PageController::AudioBible(
                                AudioBiblePage::builder()
                                    .launch((self.audio_engine.clone(), self.is_sidebar_visible))
                                    .forward(sender.input_sender(), |message| match message {
                                        AudioBibleOutput::ToggleSidebar => {
                                            AppInputMessage::ToggleSidebar
                                        }
                                    }),
                            );
                            self.pages_cache.insert(key.clone(), audio_page);
                        }
                        NavigationPage::Library(category) => {
                            let library_page = LibraryPage::builder()
                                .launch((
                                    LibraryPageCategory::from_label(category.as_str()),
                                    active_engine.clone(),
                                    self.is_sidebar_visible,
                                ))
                                .forward(sender.input_sender(), |message| match message {
                                    LibraryPageOutput::ToggleSidebar => {
                                        AppInputMessage::ToggleSidebar
                                    }
                                });
                            self.pages_cache
                                .insert(key.clone(), PageController::Library(library_page));
                        }
                        NavigationPage::Store => {}
                    }
                }

                self.current_page_key = key;
            }
        }
    }
}
