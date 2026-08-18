use adw::prelude::*;
use relm4::{ prelude::*};
use xbible_engine::engines::module_engine::module_engine_extensions::module_engine_dictionary_ext::DictionaryQuery;
use xbible_engine::engines::module_engine::module_engine_extensions::module_engine_module_content_ext::Section;
use xbible_engine::engines::module_engine::sword_module::module::SwordModule;
use xbible_engine::engines::module_engine::sword_module::module_book::ModuleBook;
use xbible_engine::engines::xbible_engine::engine::XBibleEngine;
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

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
    ExpandingThemeMenu, ThemeMenuOutput,
};
use crate::features::core::display_configurations::config::TextConfig;

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
    ProcessQueue,
    FinishedLoading,
    OpenCustomizethemePopup,
    CloseCustomizethemePopup,
    DimBackground(bool),
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
                set_margin_top: 20,

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

                    gtk::Box {
                        add_css_class: "linked",
                        add_css_class: "rounded",
                        set_halign: gtk::Align::Center,
                        set_margin_bottom: 10,

                        gtk::MenuButton {
                            set_tooltip_text: Some("Select Version"),
                            #[wrap(Some)]
                            set_child = &gtk::Box {
                                set_spacing: 4,
                                #[name = "version_label"]
                                gtk::Label { set_label: model.module.as_ref().map(|m| m.name.as_str()).unwrap_or(""),},
                                gtk::Image { set_icon_name: Some("pan-down-symbolic") },
                            },

                            #[name = "version_popover"]
                            #[wrap(Some)]
                            set_popover = &gtk::Popover {
                                set_autohide: true,
                                gtk::ScrolledWindow {
                                    set_hscrollbar_policy: gtk::PolicyType::Never,
                                    set_min_content_width: 600,
                                    set_min_content_height: 400,
                                    set_max_content_height: 600,
                                    #[name = "bible_grid"]
                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,
                                    },
                                },
                            },
                        },

                        gtk::MenuButton {
                            set_tooltip_text: Some("Select Book"),
                            #[name = "book_popover"]
                            #[wrap(Some)]
                            set_popover = &gtk::Popover {
                                gtk::ScrolledWindow {
                                    set_hscrollbar_policy: gtk::PolicyType::Never,
                                    set_min_content_width: 600,
                                    set_min_content_height: 400,
                                    set_max_content_height: 600,
                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,
                                        set_spacing: 10,
                                        set_margin_all: 12,
                                        #[name = "ot_container"]
                                        gtk::Box {
                                            set_orientation: gtk::Orientation::Vertical,
                                            set_visible: false,
                                            gtk::Label {
                                                set_label: "Old Testament",
                                                set_halign: gtk::Align::Start,
                                                add_css_class: "title-4",
                                                set_margin_bottom: 8,
                                            },
                                            #[name = "ot_grid"]
                                            gtk::FlowBox {
                                                set_max_children_per_line: 4,
                                                set_min_children_per_line: 4,
                                                set_selection_mode: gtk::SelectionMode::None,
                                                set_column_spacing: 8,
                                                set_row_spacing: 8,
                                            },
                                        },
                                        #[name = "nt_container"]
                                        gtk::Box {
                                            set_orientation: gtk::Orientation::Vertical,
                                            set_visible: false,
                                            set_margin_top: 20,
                                            gtk::Label {
                                                set_label: "New Testament",
                                                set_halign: gtk::Align::Start,
                                                add_css_class: "title-4",
                                                set_margin_bottom: 8,
                                            },
                                            #[name = "nt_grid"]
                                            gtk::FlowBox {
                                                set_max_children_per_line: 4,
                                                set_min_children_per_line: 4,
                                                set_selection_mode: gtk::SelectionMode::None,
                                                set_column_spacing: 8,
                                                set_row_spacing: 8,
                                            },
                                        },
                                    },
                                },
                            },
                            #[wrap(Some)]
                            set_child = &gtk::Box {
                                set_spacing: 4,
                                #[name = "book_label"]
                                gtk::Label { set_label: model.current_book.as_ref().map(|b| b.name.as_str()).unwrap_or("")},
                                gtk::Image { set_icon_name: Some("pan-down-symbolic") },
                            },
                        },

                        gtk::MenuButton {
                            set_tooltip_text: Some("Select Chapter"),
                            #[name = "chapter_popover"]
                            #[wrap(Some)]
                            set_popover = &gtk::Popover {
                                gtk::ScrolledWindow {
                                    set_hscrollbar_policy: gtk::PolicyType::Never,
                                    set_min_content_width: 300,
                                    set_min_content_height: 400,
                                    set_max_content_height: 600,
                                    #[name = "chapter_grid"]
                                    gtk::FlowBox {
                                        set_valign: gtk::Align::Start,
                                        set_max_children_per_line: 5,
                                        set_min_children_per_line: 5,
                                        set_selection_mode: gtk::SelectionMode::None,
                                        set_column_spacing: 6,
                                        set_row_spacing: 6,
                                        set_margin_all: 12,
                                    },
                                },
                            },
                            #[wrap(Some)]
                            set_child = &gtk::Box {
                                set_spacing: 4,
                                #[name = "chapter_label"]
                               gtk::Label { set_label: &model.current_chapter.as_ref().map(|c| format!("Chapter {c}")).unwrap_or_default(),},
                                gtk::Image { set_icon_name: Some("pan-down-symbolic") },
                            },
                        },
                    },

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

        // 3. Resolve Book & Chapter
        // We initialize these as options to safely handle the "No Module" case
        let mut active_book = None;
        let mut active_chapter = 1;
        let mut module_books = Vec::new();
        let mut chapter_count = 0;

        if let Some(ref m) = active_module {
            module_books = engine.get_books(&m.name);

            active_book = if let Some(saved_b) = &saved_state.last_book {
                module_books.iter().find(|b| &b.name == saved_b).cloned()
            } else {
                module_books.first().cloned()
            };

            if let Some(ref book) = active_book {
                chapter_count = book.chapters.len() as i32;
                active_chapter = saved_state.last_chapter.unwrap_or(1);

                if active_chapter > chapter_count {
                    active_chapter = 1;
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
            current_chapter: Some(active_chapter),
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
                },
            ),
        };

        let widgets = view_output!();

        // 6. Populate UI Components
        Self::populate_version_grid(&widgets, &modules, sender.clone());
        Self::populate_book_grid(&widgets, &module_books, sender.clone());
        Self::populate_chapter_grid(&widgets, sender.clone(), chapter_count);

        // Load initial reference

        let book = match model.current_book.clone() {
            Some(book) => book.name,
            None => String::new(),
        };
        let chapter = match model.current_chapter {
            Some(chapter) => chapter,
            None => 1,
        };
        let initial_ref = format!("{} {}", book, chapter);
        sender.input(StudyInput::LoadReference(initial_ref));

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        let start_time = std::time::Instant::now();
        match message {
            // --- ASYNC METADATA FLOW (No more engine calls here) ---
            StudyInput::SetModule(module) => {
                self.module = Some(module);
                self.current_chapter = Some(1);
                self.is_loading = true;
                if let Some(ref module) = self.module {
                    widgets.version_label.set_label(&module.name);
                } else {
                    widgets.version_label.set_label("");
                }

                // Load content for the selected module
                let reference = self
                    .current_book
                    .as_ref()
                    .map(|book| format!("{} 1", book.name))
                    .unwrap_or_default();
                sender.input(StudyInput::LoadReference(reference));
            }

            StudyInput::SetBook(book) => {
                self.current_book = Some(book.clone());
                self.current_chapter = Some(1);
                self.is_loading = true;
                if let Some(ref book) = self.current_book {
                    widgets.book_label.set_label(&book.name);
                } else {
                    widgets.book_label.set_label("");
                }
                widgets.chapter_label.set_label("Chapter 1");

                // Load content for the selected book
                let reference = self
                    .current_book
                    .as_ref()
                    .map(|book| format!("{} 1", book.name))
                    .unwrap_or_default();
                sender.input(StudyInput::LoadReference(reference));
            }

            StudyInput::SetChapter(chapter) => {
                self.current_chapter = Some(chapter);
                widgets
                    .chapter_label
                    .set_label(&format!("Chapter {}", chapter));

                self.is_loading = true;

                // Load content for the selected chapter
                let reference = self
                    .current_book
                    .as_ref()
                    .map(|book| format!("{} {}", book.name, chapter))
                    .unwrap_or_default();
                sender.input(StudyInput::LoadReference(reference));
            }

            // --- BIBLE TEXT LOADING ---
            StudyInput::LoadReference(reference) => {
                self.is_loading = true;
                widgets.loading.set_visible(self.is_loading);
                self.sections.guard().clear();

                // Fetch sections from engine
                if let Some(ref module) = self.module {
                    let sections = self.engine.get_chapter_content(&module.name, &reference);
                    sender.input(StudyInput::ReferenceLoaded(sections));
                }
            }

            StudyInput::ReferenceLoaded(sections) => {
                self.total_sections_to_load = sections.len();
                // use the "Slicer" pattern here to prevent the massive loop lag
                self.pending_sections = std::collections::VecDeque::from(sections);
                sender.input(StudyInput::ProcessQueue);
            }

            StudyInput::ProcessQueue => {
                let mut guard = self.sections.guard();

                // Add 5 sections per frame. This keeps the spinner moving.
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
                    // Done loading all widgets
                    sender.input(StudyInput::FinishedLoading);
                }
            }

            StudyInput::FinishedLoading => {
                self.is_loading = false;
                widgets.loading.set_visible(self.is_loading);
                self.total_sections_to_load = 0;
                // Attempt to find the book name from current state for saving
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

            // --- THEME & POPUP LOGIC (UNCHANGED) ---
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
        }

        let duration = start_time.elapsed();
        // This will now consistently show very small numbers (under 2ms)
        if duration.as_millis() > 10 {
            println!("Slow frame detected: {:?}", duration);
        }
        
    }
}
