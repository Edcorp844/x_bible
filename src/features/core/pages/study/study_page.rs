use adw::prelude::*;
use relm4::prelude::*;
use std::sync::{Arc, RwLock};

use crate::features::{
    bible::components::page::{
        biblepage_model::{BiblePage, StudyInput, StudyPageOutput},
        helpers::PageDisplayConfig,
    },
    core::{
        display_configurations::Config::TextConfig,
        module_engine::{
            sword_engine::SwordEngine,
            sword_module::{ModuleBook, SwordModule},
        },
    },
    dictionary::components::dictionary_model::{DictionaryInputMessage, DictionaryPage},
};

pub struct StudyPage {
    engine: Arc<SwordEngine>,
    is_sidebar_visible: bool,

    // Data Structure
    available_modules: Vec<SwordModule>,
    bible_structure: Vec<ModuleBook>,

    // UI Models
    module_list: gtk::StringList,
    book_list: gtk::StringList,
    chapter_list: gtk::StringList,

    bible_page: Controller<BiblePage>,
    dictionary_page: Controller<DictionaryPage>,
    // Selection State
    selected_module_idx: usize,
    selected_book_idx: usize,
    selected_chapter: usize,
    config: TextConfig,
}

#[derive(Debug)]
pub enum StudyPageInput {
    UpdateModule(u32),
    UpdateBook(u32),
    UpdateChapter(u32),
    LookupSelectedStrong(String),
    UpdateTheme,
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
                            #[wrap(Some)]
                            set_title_widget = &gtk::Box {
                                add_css_class: "linked",

                                // 1. MODULE DROPDOWN
                                gtk::DropDown {
                                    set_model: Some(&model.module_list),
                                    connect_selected_item_notify[sender] => move |dd| {
                                        sender.input(StudyPageInput::UpdateModule(dd.selected()));
                                    }
                                },

                                // 2. BOOK DROPDOWN
                                gtk::DropDown {
                                    #[watch]
                                    set_model: Some(&model.book_list),
                                    connect_selected_item_notify[sender] => move |dd| {
                                        sender.input(StudyPageInput::UpdateBook(dd.selected()));
                                    }
                                },

                                // 3. CHAPTER DROPDOWN
                                gtk::DropDown {
                                    #[watch]
                                    set_model: Some(&model.chapter_list),
                                    connect_selected_item_notify[sender] => move |dd| {
                                        sender.input(StudyPageInput::UpdateChapter(dd.selected()));
                                    }
                                },
                            },

                            pack_start = &gtk::ToggleButton {
                                set_icon_name: "sidebar-show-symbolic",
                                #[watch]
                                set_active: model.is_sidebar_visible,
                                connect_clicked[sender] => move |_| {
                                    let _ = sender.output(StudyPageOutPut::ToggleSidebar);
                                }
                            }
                        },

                        #[name = "main_split"]
                        #[wrap(Some)]
                        set_content = &gtk::Paned{
                            set_orientation: gtk::Orientation::Horizontal,
                            set_wide_handle: true,
                            set_shrink_start_child: true,
                            #[watch]
                            set_css_classes: &[format!("preview-area-{}", (model.config.read().unwrap().theme())).as_str(),],

                            set_start_child=Some(model.bible_page.widget()),

                            #[wrap(Some)]
                            set_end_child = &adw::ToolbarView {

                            set_margin_horizontal: 20,
                            add_top_bar: switcher_bar = &adw::InlineViewSwitcher {
                                #[watch]
                                set_stack: Some(&stack),
                                add_css_class: "round",

                            },

                            #[wrap(Some)]
                            set_content: stack = &adw::ViewStack {
                                set_vexpand: true,

                                add_titled: (
                                    &gtk::Label::new(Some("Lexicon")),
                                    Some("lex"),
                                    "Lexicon"
                                ),

                                add_titled: (
                                    model.dictionary_page.widget(),
                                    Some("dict"),
                                    "Dictionary"
                                ),

                                // Page 2: References
                                add_titled: (
                                    &gtk::Label::new(Some("Cross References")),
                                    Some("ref"),
                                    "References"
                                ),

                                add_titled: (
                                    &gtk::Label::new(Some("Commentary")),
                                    Some("Comm"),
                                    "Commentaries"
                                ),

                                add_titled: (
                                    &gtk::Label::new(Some("Compare")),
                                    Some("Comp"),
                                    "Compare"
                                ),

                                // Page 3: Lexicon

                            }
                        }

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

        // 1. Get all available Bible modules
        let available_modules = engine.get_bible_modules();
        let module_names: Vec<String> = available_modules.iter().map(|m| m.name.clone()).collect();
        let module_list =
            gtk::StringList::new(&module_names.iter().map(|s| s.as_str()).collect::<Vec<_>>());

        // 2. Load the initial structure for the first module
        let initial_module_name = module_names.first().map(|s| s.as_str()).unwrap_or("");
        let bible_structure = engine.get_bible_structure(initial_module_name);

        let book_list = gtk::StringList::new(&[]);
        let chapter_list = gtk::StringList::new(&[]);

        let bible_page = BiblePage::builder()
            .launch((
                engine.clone(),
                initial_module_name.to_string(),
                "Gen 1".to_string(),
            ))
            .forward(sender.input_sender(), move |msg| match msg {
                StudyPageOutput::ChangeTheme => StudyPageInput::UpdateTheme,
                StudyPageOutput::LookupSelectedStrong(text) => {
                    StudyPageInput::LookupSelectedStrong(text)
                }
            });

        let dictionary_page = DictionaryPage::builder().launch(engine.clone()).detach();

        let mut model = StudyPage {
            engine,
            is_sidebar_visible,
            available_modules,
            bible_structure,
            module_list,
            book_list,
            chapter_list,
            bible_page: bible_page,
            dictionary_page: dictionary_page,
            selected_module_idx: 0,
            selected_book_idx: 0,
            selected_chapter: 0,
            config: Arc::new(RwLock::new(PageDisplayConfig::new())),
        };

        // Initialize cascading lists
        model.rebuild_books();
        model.rebuild_chapters(0);

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
            StudyPageInput::UpdateModule(idx) => {
                let idx = idx as usize;
                if let Some(module) = self.available_modules.get(idx) {
                    let module_name = module.name.clone();
                    self.selected_module_idx = idx;
                    // Re-fetch the whole structure for the new module
                    self.bible_structure = self.engine.get_bible_structure(&module_name.clone());
                    self.rebuild_books();
                    self.rebuild_chapters(0);

                    self.bible_page.emit(StudyInput::SetModule(module_name));
                }
            }
            StudyPageInput::UpdateBook(idx) => {
                let idx = idx as usize;
                self.selected_book_idx = idx;
                self.rebuild_chapters(idx);
                self.bible_page
                    .emit(StudyInput::LoadReference(self.build_query_string()));
            }
            StudyPageInput::UpdateChapter(idx) => {
                self.selected_chapter = idx as usize;
                self.bible_page
                    .emit(StudyInput::LoadReference(self.build_query_string()));
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

            StudyPageInput::LookupSelectedStrong(text) => {
                self.dictionary_page
                    .emit(DictionaryInputMessage::Lookup(text));
            }
        }
    }
}

impl StudyPage {
    fn build_query_string(&self) -> String {
        if let Some(book) = self.bible_structure.get(self.selected_book_idx) {
            // Use the index + 1 for the chapter number to keep it clean,
            // or parse it from the list if the list has custom numbering.
            let chapter_number = self.selected_chapter + 1;

            // Returns something like "Genesis 1" or "John 3"
            format!("{} {}", book.name, chapter_number)
        } else {
            String::new()
        }
    }

    fn rebuild_books(&mut self) {
        let book_names: Vec<String> = self
            .bible_structure
            .iter()
            .map(|b| b.name.clone())
            .collect();
        let slice: Vec<&str> = book_names.iter().map(|s| s.as_str()).collect();
        self.book_list = gtk::StringList::new(&slice);
    }

    fn rebuild_chapters(&mut self, book_idx: usize) {
        if let Some(book) = self.bible_structure.get(book_idx) {
            let chap_strings: Vec<String> = (1..=book.chapters.len())
                .map(|i| format!("Chapter {}", i))
                .collect();

            let slice: Vec<&str> = chap_strings.iter().map(|s| s.as_str()).collect();
            self.chapter_list = gtk::StringList::new(&slice);
        } else {
            self.chapter_list = gtk::StringList::new(&[]);
        }
    }
}
