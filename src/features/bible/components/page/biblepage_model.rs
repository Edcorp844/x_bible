use adw::prelude::*;
use relm4::prelude::*;
use std::sync::{Arc, RwLock};
use xbible_engine::engines::module_engine::module_engine_extensions::module_engine_dictionary_ext::DictionaryQuery;
use xbible_engine::engines::module_engine::sword_module::module::SwordModule;
use xbible_engine::engines::module_engine::sword_module::module_book::ModuleBook;
use xbible_engine::engines::xbible_engine::engine::XBibleEngine;

use crate::features::bible::components::page::biblepage_settings::{
    BiblePageSettings, BiblePageState,
};
use crate::features::bible::components::page::helpers::PageDisplayConfig;
use crate::features::bible::components::page::section::{
    SectionInput, SectionModel, SectionOutput,
};
use crate::features::bible::components::page::verse_components::verse::VerseInputMessage;
use crate::features::bible::components::page::verse_components::verse_annotation::{
    AnnotationSettings, Annotations,
};

use crate::features::bible::components::page_theme::customize_theme_popup::{
    CustomizeThemeOutput, CustomizeThemePopup,
};
use crate::features::bible::components::page_theme::theme_button::{
    ExpandingThemeMenu, ThemeMenuInput, ThemeMenuOutput,
};
use crate::features::core::display_configurations::config::TextConfig;
use crate::features::core::pages::study::bible_page_component::biblepage_root::HeaderState;

pub struct BiblePage {
    pub(crate) engine: Arc<XBibleEngine>,
    pub(crate) module: Option<SwordModule>,
    pub(crate) sections: FactoryVecDeque<SectionModel>,
    pub(crate) config: TextConfig,
    pub(crate) customize_theme_popup: Option<Controller<CustomizeThemePopup>>,
    pub(crate) annotations: Annotations,

    pub(crate) current_book: Option<ModuleBook>,
    pub(crate) current_chapter: Option<i32>,
    pub(crate) is_loading: bool,
    pub(crate) expanding_theme_menu: Controller<ExpandingThemeMenu>,
}

#[derive(Debug)]
pub enum StudyInput {
    LoadReference(String),
    SetModule(SwordModule),
    SetBook(ModuleBook),
    SetChapter(i32),
    ToggleDisplay(VerseInputMessage),
    SetConfig(TextConfig),
    HeaderStateChanged(HeaderState),
    FinishedLoading,
    OpenCustomizethemePopup,
    CloseCustomizethemePopup,
    DimBackground(bool),
    NextChapter,
    PreviousChapter,
}

#[derive(Debug)]
pub enum StudyPageOutput {
    ChangeTheme,
    LookupSelectedStrong(DictionaryQuery),
}

#[relm4::component(pub)]
impl Component for BiblePage {
    type Init = Arc<XBibleEngine>;
    type Input = StudyInput;
    type Output = StudyPageOutput;
    type CommandOutput = ();

    view! {
        adw::NavigationPage {
            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                #[name="page_overlay"]
                gtk::Overlay {
                    set_vexpand: true,
                    #[watch]
                    set_css_classes: &[
                        "page-overlay",
                        &model.make_css_preview_clss(model.config.read().unwrap().theme())
                    ],

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_halign: gtk::Align::Fill,
                        set_hexpand: true,

                        #[name="loading"]
                        gtk::Box {
                            #[watch]
                            set_visible: model.is_loading,
                            set_height_request: 2,
                            set_halign: gtk::Align::Fill,
                            set_hexpand: true,
                            add_css_class: "loading-line-pulse",
                        },

                        gtk::ScrolledWindow {
                            set_vexpand: true,
                            set_hexpand: true,
                            set_hscrollbar_policy: gtk::PolicyType::Never,

                            #[local_ref]
                            section_list -> gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_margin_all: 30,
                            },
                        },
                    },

                    #[name = "dim_scrim"]
                    add_overlay = &gtk::Box {
                        add_css_class: "dim-scrim",
                        set_visible: false,
                        set_can_target: false,
                    },

                    add_overlay = model.expanding_theme_menu.widget() {
                        set_margin_all: 25,
                    },
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        info!("[BiblePage] Initializing BiblePage component");
        let engine = init;

        // 1. Setup Sections Factory
        let section_list = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let sections = FactoryVecDeque::builder()
            .launch(section_list.clone())
            .forward(sender.output_sender(), move |message| match message {
                SectionOutput::Lookup(query) => {
                    debug!("[BiblePage] Forwarding Strong's lookup query: {:?}", query);
                    StudyPageOutput::LookupSelectedStrong(query)
                }
            });

        // 2. Load Saved Session / Resolve Active Module
        let saved_state = BiblePageSettings::load();
        debug!("[BiblePage] Loaded saved state: {:?}", saved_state);
        let modules = engine.get_bible_modules();
        info!("[BiblePage] Retrieved {} Bible modules", modules.len());

        let active_module = if let Some(saved_name) = saved_state.last_module {
            modules.iter().find(|m| m.name == saved_name).cloned()
        } else {
            modules.first().cloned()
        };

        // 3. Resolve Book & Chapter Dynamically
        let mut active_book = None;
        let mut active_chapter = None;
        let mut module_books = Vec::new();

        if let Some(ref m) = active_module {
            info!("[BiblePage] Active module set to: {}", m.name);
            module_books = engine.get_books(&m.name);
            debug!("[BiblePage] Module {} contains {} books", m.name, module_books.len());

            active_book = saved_state
                .last_book
                .as_ref()
                .and_then(|saved_b| module_books.iter().find(|b| &b.name == saved_b).cloned())
                .or_else(|| module_books.first().cloned());

            if let Some(ref book) = active_book {
                if let Some(saved_c) = saved_state.last_chapter {
                    if book.chapters.iter().any(|c| c.number == saved_c) {
                        active_chapter = Some(saved_c);
                    }
                }
                if active_chapter.is_none() {
                    active_chapter = book.chapters.first().map(|c| c.number);
                }
            }
        } else {
            warn!("[BiblePage] No SWORD modules found! Please install a SWORD module.");
        }

        let config = Arc::new(RwLock::new(PageDisplayConfig::new()));
        let annotations = AnnotationSettings::load_all();
        debug!("[BiblePage] Annotations loaded successfully");

        let model = BiblePage {
            engine,
            module: active_module,
            current_book: active_book,
            current_chapter: active_chapter,
            sections,
            config: config.clone(),
            customize_theme_popup: None,
            annotations,
            is_loading: false,
            expanding_theme_menu: ExpandingThemeMenu::builder().launch(config).forward(
                sender.input_sender(),
                |output| match output {
                    ThemeMenuOutput::OpenThemePopup => StudyInput::OpenCustomizethemePopup,
                    ThemeMenuOutput::ToggleDisplay(msg) => StudyInput::ToggleDisplay(msg),
                    ThemeMenuOutput::DimBackground(dim) => StudyInput::DimBackground(dim),
                    ThemeMenuOutput::NextChapter => StudyInput::NextChapter,
                    ThemeMenuOutput::PreviousChapter => StudyInput::PreviousChapter,
                },
            ),
        };

        let widgets = view_output!();

        model.update_nav_sensitivity();

        // Load initial content directly
        if let (Some(book), Some(chapter)) = (&model.current_book, model.current_chapter) {
            let initial_ref = format!("{} {}", book.name, chapter);
            info!("[BiblePage] Triggering initial reference load: {}", initial_ref);
            sender.input(StudyInput::LoadReference(initial_ref));
        }

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        debug!("[BiblePage] Received StudyInput message: {:?}", message);

        match message {
            StudyInput::SetModule(module) => {
                info!("[BiblePage] Switching module to: {}", module.name);
                let books = self.engine.get_books(&module.name);
                self.module = Some(module);

                self.current_book = books.first().cloned();
                self.current_chapter = self
                    .current_book
                    .as_ref()
                    .and_then(|b| b.chapters.first().map(|c| c.number));

                self.update_nav_sensitivity();
                self.load_current_reference(sender);
            }

            StudyInput::SetBook(book) => {
                info!("[BiblePage] Switching book to: {}", book.name);
                self.current_book = Some(book.clone());
                self.current_chapter = book.chapters.first().map(|c| c.number);

                self.update_nav_sensitivity();
                self.load_current_reference(sender);
            }

            StudyInput::SetChapter(chapter) => {
                info!("[BiblePage] Switching chapter to: {}", chapter);
                self.current_chapter = Some(chapter);

                self.update_nav_sensitivity();
                self.load_current_reference(sender);
            }

            StudyInput::NextChapter => {
                debug!("[BiblePage] Navigating to Next Chapter");
                let (Some(module), Some(curr_book), Some(curr_chap)) = (
                    self.module.as_ref(),
                    self.current_book.as_ref(),
                    self.current_chapter,
                ) else {
                    warn!("[BiblePage] NextChapter triggered without active module/book/chapter");
                    return;
                };

                let books = self.engine.get_books(&module.name);
                let Some(b_idx) = books.iter().position(|b| b.name == curr_book.name) else {
                    return;
                };

                if let Some(c_idx) = curr_book.chapters.iter().position(|c| c.number == curr_chap) {
                    if c_idx + 1 < curr_book.chapters.len() {
                        let next_chap = curr_book.chapters[c_idx + 1].number;
                        sender.input(StudyInput::SetChapter(next_chap));
                    } else if b_idx + 1 < books.len() {
                        let next_book = books[b_idx + 1].clone();
                        sender.input(StudyInput::SetBook(next_book));
                    }
                }
            }

            StudyInput::PreviousChapter => {
                debug!("[BiblePage] Navigating to Previous Chapter");
                let (Some(module), Some(curr_book), Some(curr_chap)) = (
                    self.module.as_ref(),
                    self.current_book.as_ref(),
                    self.current_chapter,
                ) else {
                    warn!("[BiblePage] PreviousChapter triggered without active module/book/chapter");
                    return;
                };

                let books = self.engine.get_books(&module.name);
                let Some(b_idx) = books.iter().position(|b| b.name == curr_book.name) else {
                    return;
                };

                if let Some(c_idx) = curr_book.chapters.iter().position(|c| c.number == curr_chap) {
                    if c_idx > 0 {
                        let prev_chap = curr_book.chapters[c_idx - 1].number;
                        sender.input(StudyInput::SetChapter(prev_chap));
                    } else if b_idx > 0 {
                        let prev_book = books[b_idx - 1].clone();
                        let last_chap = prev_book.chapters.last().map(|c| c.number).unwrap_or(1);

                        self.current_book = Some(prev_book.clone());
                        self.current_chapter = Some(last_chap);

                        self.update_nav_sensitivity();
                        self.load_current_reference(sender);
                    }
                }
            }

            StudyInput::LoadReference(reference) => {
                info!("[BiblePage] Loading reference: {}", reference);
                self.is_loading = true;
                widgets.loading.set_visible(self.is_loading);

                // Clear previous Factory items cleanly without scheduling idle callbacks
                let mut guard = self.sections.guard();
                let prev_count = guard.len();
                guard.clear();
                debug!("[BiblePage] Cleared {} existing section widgets from FactoryVecDeque", prev_count);

                if let Some(ref module) = self.module {
                    debug!("[BiblePage] Querying C FFI for reference '{}' in module '{}'", reference, module.name);
                    let sections = self.engine.get_chapter_content(&module.name, &reference);
                    info!("[BiblePage] Retrieved {} sections from engine. Rendering to Factory...", sections.len());
                    //let sections = Vec::new(); // Placeholder for actual section retrieval logic
                    // Directly populate factory in one pass to avoid GLib idle task loops
                    for section in sections {
                        guard.push_back((
                            section,
                            self.config.clone(),
                            self.annotations.clone(),
                        ));
                    }
                } else {
                    warn!("[BiblePage] Cannot load reference '{}': No module set", reference);
                }

                sender.input(StudyInput::FinishedLoading);
            }

            StudyInput::FinishedLoading => {
                info!("[BiblePage] Finished loading reference. Resetting loading indicator.");
                self.is_loading = false;
                widgets.loading.set_visible(self.is_loading);

                if let (Some(module), Some(book)) =
                    (self.module.as_ref(), self.current_book.as_ref())
                {
                    debug!("[BiblePage] Saving state: Module={}, Book={}, Chapter={:?}", module.name, book.name, self.current_chapter);
                    BiblePageSettings::save(BiblePageState {
                        last_module: Some(module.name.clone()),
                        last_book: Some(book.name.clone()),
                        last_chapter: self.current_chapter,
                    });
                }
            }

            StudyInput::OpenCustomizethemePopup => {
                info!("[BiblePage] Opening Theme Customization Popup");
                let controller = CustomizeThemePopup::builder()
                    .launch((self.config.clone(), self.engine.clone()))
                    .forward(sender.input_sender(), |msg| match msg {
                        CustomizeThemeOutput::Close => StudyInput::CloseCustomizethemePopup,
                        CustomizeThemeOutput::SaveConfig(config) => StudyInput::SetConfig(config),
                    });
                let popup_window = controller.widget();

                if let Some(root_window) = relm4::main_application().active_window() {
                    popup_window.set_transient_for(Some(&root_window));
                }
                popup_window.set_modal(true);
                popup_window.present();

                self.customize_theme_popup = Some(controller);
            }

            StudyInput::CloseCustomizethemePopup => {
                info!("[BiblePage] Closing Theme Customization Popup");
                if let Some(c) = self.customize_theme_popup.take() {
                    c.widget().close();
                }
            }

            StudyInput::ToggleDisplay(msg) => {
                debug!("[BiblePage] Applying display message: {:?}", msg);
                self.config.write().unwrap().apply_message(&msg);
                for i in 0..self.sections.len() {
                    self.sections
                        .send(i, SectionInput::ToggleDisplay(msg.clone()));
                }
                let class = self.make_css_preview_clss(self.config.read().unwrap().theme());
                let _ = sender.output(StudyPageOutput::ChangeTheme);
                widgets
                    .page_overlay
                    .set_css_classes(&["page-overlay", &class]);
            }

            StudyInput::SetConfig(config) => {
                debug!("[BiblePage] Setting new text configuration");
                sender.input(StudyInput::ToggleDisplay(
                    VerseInputMessage::UpdateDisplayConf(config),
                ));
            }

            StudyInput::DimBackground(dim) => {
                widgets.dim_scrim.set_visible(dim);
            }

            StudyInput::HeaderStateChanged(state) => {
                info!("[BiblePage] Header state changed: Module={:?}, Book={:?}, Chapter={:?}", 
                    state.module.as_ref().map(|m| &m.name), 
                    state.book.as_ref().map(|b| &b.name), 
                    state.chapter
                );
                self.module = state.module.clone();
                self.current_book = state.book.clone();
                self.current_chapter = state.chapter;
                self.load_current_reference(sender);
            }
        }
    }
}

impl BiblePage {
    fn load_current_reference(&mut self, sender: ComponentSender<Self>) {
        if let (Some(book), Some(chap)) = (&self.current_book, self.current_chapter) {
            let reference = format!("{} {}", book.name, chap);
            debug!("[BiblePage] Requesting reference load for: {}", reference);
            sender.input(StudyInput::LoadReference(reference));
        } else {
            warn!("[BiblePage] Cannot load reference: Book or chapter missing (Book: {:?}, Chapter: {:?})", self.current_book, self.current_chapter);
        }
    }

    fn update_nav_sensitivity(&self) {
        let (Some(module), Some(curr_book), Some(curr_chap)) = (
            self.module.as_ref(),
            self.current_book.as_ref(),
            self.current_chapter,
        ) else {
            self.expanding_theme_menu.emit(ThemeMenuInput::SetNavigationState {
                has_prev: false,
                has_next: false,
            });
            return;
        };

        let books = self.engine.get_books(&module.name);
        let Some(b_idx) = books.iter().position(|b| b.name == curr_book.name) else {
            return;
        };
        let Some(c_idx) = curr_book.chapters.iter().position(|c| c.number == curr_chap) else {
            return;
        };

        let is_first_book = b_idx == 0;
        let is_last_book = b_idx + 1 == books.len();
        let is_first_chap = c_idx == 0;
        let is_last_chap = c_idx + 1 == curr_book.chapters.len();

        let has_prev = !(is_first_book && is_first_chap);
        let has_next = !(is_last_book && is_last_chap);

        debug!("[BiblePage] Updating nav sensitivity: HasPrev={}, HasNext={}", has_prev, has_next);
        self.expanding_theme_menu.emit(ThemeMenuInput::SetNavigationState {
            has_prev,
            has_next,
        });
    }
}