use adw::prelude::*;
use relm4::prelude::*;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use xbible_engine::engines::module_engine::module_engine_extensions::module_engine_dictionary_ext::DictionaryQuery;
use xbible_engine::engines::module_engine::module_engine_extensions::module_engine_module_content_ext::Section;
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

    pub(crate) pending_sections: VecDeque<Section>,
    pub(crate) total_sections_to_load: usize,

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
    ReferenceLoaded(Vec<Section>),
    HeaderStateChanged(HeaderState),
    ProcessQueue,
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

                add_overlay = model.expanding_theme_menu.widget(){
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
        let engine = init;

        // 1. Setup Sections Factory
        let section_list = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let sections = FactoryVecDeque::builder()
            .launch(section_list.clone())
            .forward(sender.output_sender(), move |message| match message {
                SectionOutput::Lookup(query) => StudyPageOutput::LookupSelectedStrong(query),
            });

        // 2. Load Saved Session / Resolve Active Module
        let saved_state = BiblePageSettings::load();
        let modules = engine.get_bible_modules();

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
            module_books = engine.get_books(&m.name);

            // Attempt saved book, or fallback to first book in module
            active_book = saved_state
                .last_book
                .as_ref()
                .and_then(|saved_b| module_books.iter().find(|b| &b.name == saved_b).cloned())
                .or_else(|| module_books.first().cloned());

            if let Some(ref book) = active_book {
                // Find matching chapter or fallback to book's first chapter
                if let Some(saved_c) = saved_state.last_chapter {
                    if book.chapters.iter().any(|c| c.number == saved_c) {
                        active_chapter = Some(saved_c);
                    }
                }
                if active_chapter.is_none() {
                    active_chapter = book.chapters.first().map(|c| c.number);
                }
            }
        }

        if active_module.is_none() {
            eprintln!("No modules found. Please install a SWORD module.");
        }

        let config = Arc::new(RwLock::new(PageDisplayConfig::new()));
        let model = BiblePage {
            engine,
            module: active_module,
            current_book: active_book,
            current_chapter: active_chapter,
            sections,
            config: config.clone(),
            customize_theme_popup: None,
            annotations: AnnotationSettings::load_all(),
            is_loading: false,
            pending_sections: VecDeque::new(),
            total_sections_to_load: 0,
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

        // Populate initial UI widgets
        // Self::populate_version_grid(&widgets, &modules, sender.clone());
        // Self::populate_book_grid(&widgets, &module_books, sender.clone());
        // let chapter_count = model.current_book.as_ref().map(|b| b.chapters.len() as i32).unwrap_or(0);
        // Self::populate_chapter_grid(&widgets, sender.clone(), chapter_count);

        // Notify theme menu of initial navigation sensitivity
        model.update_nav_sensitivity();

        // Load initial content
        if let (Some(book), Some(chapter)) = (&model.current_book, model.current_chapter) {
            let initial_ref = format!("{} {}", book.name, chapter);
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
        match message {
            StudyInput::SetModule(module) => {
                let books = self.engine.get_books(&module.name);
                self.module = Some(module);
                
                // Fallback to first book and first chapter of new module
                self.current_book = books.first().cloned();
                self.current_chapter = self
                    .current_book
                    .as_ref()
                    .and_then(|b| b.chapters.first().map(|c| c.number));

                // widgets.version_label.set_label(
                //     self.module.as_ref().map(|m| m.name.as_str()).unwrap_or(""),
                // );

                // Update dependent popovers
                let modules = self.engine.get_bible_modules();
                // Self::populate_version_grid(widgets, &modules, sender.clone());
                // Self::populate_book_grid(widgets, &books, sender.clone());

                // let chapter_count = self.current_book.as_ref().map(|b| b.chapters.len() as i32).unwrap_or(0);
                // Self::populate_chapter_grid(widgets, sender.clone(), chapter_count);

                self.update_nav_sensitivity();
                self.load_current_reference(sender);
            }

            StudyInput::SetBook(book) => {
                self.current_book = Some(book.clone());
                self.current_chapter = book.chapters.first().map(|c| c.number);

                // widgets.book_label.set_label(&book.name);
                // widgets.chapter_label.set_label(
                //     &self
                //         .current_chapter
                //         .map(|c| format!("Chapter {c}"))
                //         .unwrap_or_default(),
                // );

                // Self::populate_chapter_grid(widgets, sender.clone(), book.chapters.len() as i32);
                self.update_nav_sensitivity();
                self.load_current_reference(sender);
            }

            StudyInput::SetChapter(chapter) => {
                self.current_chapter = Some(chapter);
                //widgets.chapter_label.set_label(&format!("Chapter {}", chapter));

                self.update_nav_sensitivity();
                self.load_current_reference(sender);
            }

            StudyInput::NextChapter => {
                let (Some(module), Some(curr_book), Some(curr_chap)) = (
                    self.module.as_ref(),
                    self.current_book.as_ref(),
                    self.current_chapter,
                ) else {
                    return;
                };

                let books = self.engine.get_books(&module.name);
                let Some(b_idx) = books.iter().position(|b| b.name == curr_book.name) else {
                    return;
                };

                if let Some(c_idx) = curr_book.chapters.iter().position(|c| c.number == curr_chap) {
                    if c_idx + 1 < curr_book.chapters.len() {
                        // Move to next chapter in current book
                        let next_chap = curr_book.chapters[c_idx + 1].number;
                        sender.input(StudyInput::SetChapter(next_chap));
                    } else if b_idx + 1 < books.len() {
                        // Move to first chapter of next book
                        let next_book = books[b_idx + 1].clone();
                        sender.input(StudyInput::SetBook(next_book));
                    }
                }
            }

            StudyInput::PreviousChapter => {
                let (Some(module), Some(curr_book), Some(curr_chap)) = (
                    self.module.as_ref(),
                    self.current_book.as_ref(),
                    self.current_chapter,
                ) else {
                    return;
                };

                let books = self.engine.get_books(&module.name);
                let Some(b_idx) = books.iter().position(|b| b.name == curr_book.name) else {
                    return;
                };

                if let Some(c_idx) = curr_book.chapters.iter().position(|c| c.number == curr_chap) {
                    if c_idx > 0 {
                        // Move to previous chapter in current book
                        let prev_chap = curr_book.chapters[c_idx - 1].number;
                        sender.input(StudyInput::SetChapter(prev_chap));
                    } else if b_idx > 0 {
                        // Move to last chapter of previous book
                        let prev_book = books[b_idx - 1].clone();
                        let last_chap = prev_book.chapters.last().map(|c| c.number).unwrap_or(1);
                        
                        self.current_book = Some(prev_book.clone());
                        self.current_chapter = Some(last_chap);

                        // widgets.book_label.set_label(&prev_book.name);
                        // widgets.chapter_label.set_label(&format!("Chapter {}", last_chap));

                        // Self::populate_chapter_grid(widgets, sender.clone(), prev_book.chapters.len() as i32);
                        self.update_nav_sensitivity();
                        self.load_current_reference(sender);
                    }
                }
            }

            StudyInput::LoadReference(reference) => {
                self.is_loading = true;
                widgets.loading.set_visible(self.is_loading);
                self.sections.guard().clear();

                if let Some(ref module) = self.module {
                    let sections = self.engine.get_chapter_content(&module.name, &reference);
                    sender.input(StudyInput::ReferenceLoaded(sections));
                }
            }

            StudyInput::ReferenceLoaded(sections) => {
                self.total_sections_to_load = sections.len();
                self.pending_sections = VecDeque::from(sections);
                sender.input(StudyInput::ProcessQueue);
            }

            StudyInput::ProcessQueue => {
                let mut guard = self.sections.guard();

                for _ in 0..5 {
                    if let Some(section) = self.pending_sections.pop_front() {
                        guard.push_back((section, self.config.clone(), self.annotations.clone()));
                    }
                }

                if !self.pending_sections.is_empty() {
                    let sender = sender.clone();
                    glib::idle_add_local(move || {
                        sender.input(StudyInput::ProcessQueue);
                        glib::ControlFlow::Break
                    });
                } else {
                    sender.input(StudyInput::FinishedLoading);
                }
            }

            StudyInput::FinishedLoading => {
                self.is_loading = false;
                widgets.loading.set_visible(self.is_loading);
                self.total_sections_to_load = 0;

                if let (Some(module), Some(book)) =
                    (self.module.as_ref(), self.current_book.as_ref())
                {
                    BiblePageSettings::save(BiblePageState {
                        last_module: Some(module.name.clone()),
                        last_book: Some(book.name.clone()),
                        last_chapter: self.current_chapter,
                    });
                }
            }

            StudyInput::OpenCustomizethemePopup => {
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
                if let Some(c) = self.customize_theme_popup.take() {
                    c.widget().close();
                }
            }

            StudyInput::ToggleDisplay(msg) => {
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
                sender.input(StudyInput::ToggleDisplay(
                    VerseInputMessage::UpdateDisplayConf(config),
                ));
            }

            StudyInput::DimBackground(dim) => {
                widgets.dim_scrim.set_visible(dim);
            }
            StudyInput::HeaderStateChanged(state) => {
                self.module = state.module.clone();
                self.current_book = state.book.clone();
                self.current_chapter = state.chapter;
                self.load_current_reference(sender);

            },
        }
    }
}

impl BiblePage {
    fn load_current_reference(&mut self, sender: ComponentSender<Self>) {
        if let (Some(book), Some(chap)) = (&self.current_book, self.current_chapter) {
            sender.input(StudyInput::LoadReference(format!("{} {}", book.name, chap)));
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

        self.expanding_theme_menu.emit(ThemeMenuInput::SetNavigationState {
            has_prev,
            has_next,
        });
    }
}