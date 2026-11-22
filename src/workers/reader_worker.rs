use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use relm4::prelude::ComponentSender;
use xbible_engine::features::reader::search::{
    SearchFilter, SearchMode, SearchPhase, SearchResults,
};
use xbible_engine::features::reader::{
    BookSection, CommentaryChapter, EntityRef, ReaderChapter, XBibleReader,
};
use xbible_engine::features::store::{Store, StoreItem};
use xbible_engine::sources::models::{GenericBook, GenericTranslation};

use crate::core::engine::SharedEngine;

const THEOGRAPHIC_DATASET: &str = "theographic";
const DEFAULT_SEARCH_LIMIT: i32 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum CanonSection {
    Ot,
    Nt,
    Deutero,
    OrthodoxExtra,
    EthiopianExtra,
    Unknown,
}

impl CanonSection {
    pub const ALL: [Self; 6] = [
        Self::Ot,
        Self::Nt,
        Self::Deutero,
        Self::OrthodoxExtra,
        Self::EthiopianExtra,
        Self::Unknown,
    ];

    pub fn wire(&self) -> &'static str {
        match self {
            Self::Ot => "ot",
            Self::Nt => "nt",
            Self::Deutero => "deutero",
            Self::OrthodoxExtra => "orthodox_extra",
            Self::EthiopianExtra => "ethiopian_extra",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_wire(s: &str) -> Self {
        match s.trim().to_lowercase().as_str() {
            "ot" | "old_testament" | "oldtestament" => Self::Ot,
            "nt" | "new_testament" | "newtestament" => Self::Nt,
            "deutero" | "deuterocanon" | "apocrypha" => Self::Deutero,
            "orthodox_extra" | "orthodoxextra" => Self::OrthodoxExtra,
            "ethiopian_extra" | "ethiopianextra" => Self::EthiopianExtra,
            "unknown" | "" => Self::Unknown,
            _ => Self::Unknown,
        }
    }

    pub fn all_set() -> HashSet<Self> {
        Self::ALL.into_iter().collect()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReaderTabKind {
    Read,
    Search,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SearchType {
    RegularExpression,
    Phrase,
    MultiWord,
}

impl SearchType {
    pub fn engine_mode(&self) -> SearchMode {
        match self {
            Self::RegularExpression => SearchMode::Regex,
            Self::Phrase => SearchMode::Phrase,
            Self::MultiWord => SearchMode::MultiWord,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ReaderTabState {
    pub id: String,
    pub kind: ReaderTabKind,
    pub translation: Option<GenericTranslation>,
    pub commentary_id: Option<String>,
    pub commentary_name: String,
    pub commentary_data: Option<CommentaryChapter>,
    pub sections: Vec<BookSection>,
    pub book_id: Option<String>,
    pub book_name: String,
    pub chapter: i32,
    pub chapter_data: Option<ReaderChapter>,
    pub search_query: String,
    pub search_type: SearchType,
    pub selected_sections: HashSet<CanonSection>,
    pub is_searching: bool,
    pub search_results: Option<SearchResults>,
    pub search_error: Option<String>,
}

impl ReaderTabState {
    pub fn new_read_tab(id: String, translation: Option<GenericTranslation>) -> Self {
        Self {
            id,
            kind: ReaderTabKind::Read,
            translation,
            commentary_id: None,
            commentary_name: String::new(),
            commentary_data: None,
            sections: Vec::new(),
            book_id: None,
            book_name: String::new(),
            chapter: 1,
            chapter_data: None,
            search_query: String::new(),
            search_type: SearchType::MultiWord,
            selected_sections: CanonSection::all_set(),
            is_searching: false,
            search_results: None,
            search_error: None,
        }
    }

    pub fn new_search_tab(id: String, translation: Option<GenericTranslation>) -> Self {
        let mut tab = Self::new_read_tab(id, translation);
        tab.kind = ReaderTabKind::Search;
        tab
    }

    pub fn books(&self) -> Vec<GenericBook> {
        self.sections
            .iter()
            .flat_map(|sec| sec.books.iter().cloned())
            .collect()
    }

    pub fn is_commentary(&self) -> bool {
        self.commentary_id.is_some() && self.translation.is_none()
    }
}

#[derive(Debug)]
pub(crate) struct ReaderWorker {
    engine: SharedEngine,
    reader: Arc<XBibleReader>,
    store: Arc<Store>,

    is_ready: bool,
    is_loading_content: bool,
    section_update_required: bool,

    installed_translations: Vec<GenericTranslation>,
    current_translation: Option<GenericTranslation>,
    parallel_translation: Option<GenericTranslation>,

    current_chapter_data: Option<ReaderChapter>,
    parallel_chapter_data: Option<ReaderChapter>,
    current_sections: Vec<BookSection>,
    chapter_entities: Vec<EntityRef>,
    verse_entities: Vec<EntityRef>,

    commentaries: Vec<StoreItem>,
    current_commentary_id: Option<String>,
    current_commentary_name: String,
    current_commentary_data: Option<CommentaryChapter>,

    study_commentary_id: Option<String>,
    study_commentary_name: String,
    study_commentary_data: Option<CommentaryChapter>,
    study_commentary_loading: bool,

    tabs: Vec<ReaderTabState>,
    active_tab_index: usize,
    next_tab_seq: usize,

    search_generation: u64,
    search_phase: SearchPhase,
    search_fraction: f64,
    search_message: String,
}

#[derive(Debug)]
pub(crate) enum ReaderWorkerInput {
    Setup,
    ReloadInstalledTranslations,
    ReloadCommentaries,

    SelectTranslation(GenericTranslation),
    SelectParallelTranslation(Option<GenericTranslation>),
    OpenChapter {
        book_id: String,
        chapter: i32,
    },
    SelectBook(String),
    NextChapter,
    PreviousChapter,

    SetSelectedCanon(String),
    LoadEntitiesForVerse {
        book_id: String,
        chapter: i32,
        verse: i32,
    },

    OpenCommentaryChapter {
        commentary_id: String,
        commentary_name: String,
        book_id: String,
        chapter: i32,
    },
    OpenCommentaryForCurrentChapter {
        commentary_id: String,
        commentary_name: String,
    },
    SetStudyCommentary {
        id: String,
        name: String,
    },
    ClearStudyCommentary,
    RefreshStudyCommentary,

    SwitchTab(usize),
    AddNewTab,
    AddSearchTab,
    RemoveTab(usize),

    SetSearchQuery(String),
    SetSearchType(SearchType),
    ToggleSearchSection(CanonSection),
    PerformSearch(String),
    ClearSearch,

    // ── Internal async completion events ──────────────────────────────────
    #[doc(hidden)]
    InstalledTranslationsLoaded(Vec<GenericTranslation>),
    #[doc(hidden)]
    CommentariesLoaded(Vec<StoreItem>),
    #[doc(hidden)]
    SectionsLoaded {
        translation_id: String,
        sections: Vec<BookSection>,
        needs_update: bool,
        open_first_chapter: bool,
    },
    #[doc(hidden)]
    ChapterLoaded {
        translation: GenericTranslation,
        book_id: String,
        chapter: i32,
        data: Option<ReaderChapter>,
        parallel_data: Option<ReaderChapter>,
    },
    #[doc(hidden)]
    ChapterEntitiesLoaded {
        book_id: String,
        chapter: i32,
        entities: Vec<EntityRef>,
    },
    #[doc(hidden)]
    VerseEntitiesLoaded(Vec<EntityRef>),
    #[doc(hidden)]
    CommentaryChapterLoaded {
        commentary_id: String,
        commentary_name: String,
        book_id: String,
        chapter: i32,
        result: Result<Option<CommentaryChapter>, String>,
    },
    #[doc(hidden)]
    StudyCommentaryLoaded(Result<Option<CommentaryChapter>, String>),
    #[doc(hidden)]
    SearchCompleted {
        generation: u64,
        tab_index: usize,
        result: Result<SearchResults, String>,
    },
}

#[derive(Debug, Clone)]
pub(crate) enum ReaderWorkerOutput {
    ReadyChanged(bool),
    LoadingContentChanged(bool),
    Error(String),
    InstalledTranslationsUpdated(Vec<GenericTranslation>),
    CommentariesUpdated(Vec<StoreItem>),
    SectionUpdateRequiredChanged(bool),
    SectionsUpdated(Vec<BookSection>),
    ChapterChanged {
        chapter: Option<ReaderChapter>,
        parallel: Option<ReaderChapter>,
        translation: Option<GenericTranslation>,
    },
    ChapterEntitiesUpdated(Vec<EntityRef>),
    VerseEntitiesUpdated(Vec<EntityRef>),
    CommentaryChanged {
        commentary_id: Option<String>,
        commentary_name: String,
        data: Option<CommentaryChapter>,
    },
    StudyCommentaryUpdated {
        id: Option<String>,
        name: String,
        data: Option<CommentaryChapter>,
        loading: bool,
    },
    SearchProgress {
        phase: SearchPhase,
        fraction: f64,
        message: String,
    },
    SearchResultsUpdated {
        tab_index: usize,
        is_searching: bool,
        results: Option<SearchResults>,
        error: Option<String>,
    },
    TabsUpdated {
        tabs: Vec<ReaderTabState>,
        active_index: usize,
    },
}

impl relm4::Worker for ReaderWorker {
    type Init = SharedEngine;
    type Input = ReaderWorkerInput;
    type Output = ReaderWorkerOutput;

    fn init(engine: Self::Init, sender: ComponentSender<Self>) -> Self {
        let reader = engine.reader();
        let store = engine.store();
        let initial_tab = ReaderTabState::new_read_tab("tab-1".to_string(), None);

        let worker = Self {
            engine,
            reader,
            store,
            is_ready: true,
            is_loading_content: false,
            section_update_required: false,
            installed_translations: Vec::new(),
            current_translation: None,
            parallel_translation: None,
            current_chapter_data: None,
            parallel_chapter_data: None,
            current_sections: Vec::new(),
            chapter_entities: Vec::new(),
            verse_entities: Vec::new(),
            commentaries: Vec::new(),
            current_commentary_id: None,
            current_commentary_name: String::new(),
            current_commentary_data: None,
            study_commentary_id: None,
            study_commentary_name: String::new(),
            study_commentary_data: None,
            study_commentary_loading: false,
            tabs: vec![initial_tab],
            active_tab_index: 0,
            next_tab_seq: 1,
            search_generation: 0,
            search_phase: SearchPhase::Done,
            search_fraction: 0.0,
            search_message: String::new(),
        };

        let _ = sender.output(ReaderWorkerOutput::ReadyChanged(true));
        sender.input(ReaderWorkerInput::Setup);
        worker
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            ReaderWorkerInput::Setup => {
                debug!("[ReaderWorker] setup: loading translations and commentaries");
                sender.input(ReaderWorkerInput::ReloadInstalledTranslations);
                sender.input(ReaderWorkerInput::ReloadCommentaries);
            }

            ReaderWorkerInput::ReloadInstalledTranslations => {
                let reader = Arc::clone(&self.reader);
                let tx = sender.input_sender().clone();
                std::thread::spawn(move || {
                    let translations = reader.get_installed_translations();
                    let _ = tx.send(ReaderWorkerInput::InstalledTranslationsLoaded(translations));
                });
            }

            ReaderWorkerInput::InstalledTranslationsLoaded(translations) => {
                self.installed_translations = translations.clone();
                let _ = sender.output(ReaderWorkerOutput::InstalledTranslationsUpdated(
                    translations,
                ));

                if self.installed_translations.is_empty() {
                    debug!("[ReaderWorker] no translations installed yet");
                    return;
                }

                let target_translation = self
                    .current_translation
                    .clone()
                    .filter(|current| {
                        self.installed_translations
                            .iter()
                            .any(|item| item.id == current.id)
                    })
                    .or_else(|| self.installed_translations.first().cloned());

                if let Some(translation) = target_translation {
                    self.current_translation = Some(translation.clone());
                    if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
                        tab.translation = Some(translation.clone());
                        tab.commentary_id = None;
                        tab.commentary_name.clear();
                    }
                    self.request_sections(translation.id.clone(), true, &sender);
                }
            }

            ReaderWorkerInput::ReloadCommentaries => {
                let store = Arc::clone(&self.store);
                let tx = sender.input_sender().clone();
                std::thread::spawn(move || {
                    let shelves = store.home_feed(None);
                    let mut by_id: HashMap<String, StoreItem> = HashMap::new();
                    for shelf in shelves {
                        for item in shelf.items {
                            if item.kind == "commentary" && item.installed {
                                by_id.insert(item.id.clone(), item);
                            }
                        }
                    }
                    let mut items: Vec<StoreItem> = by_id.into_values().collect();
                    items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                    let _ = tx.send(ReaderWorkerInput::CommentariesLoaded(items));
                });
            }

            ReaderWorkerInput::CommentariesLoaded(items) => {
                self.commentaries = items.clone();
                let _ = sender.output(ReaderWorkerOutput::CommentariesUpdated(items));
            }

            ReaderWorkerInput::SectionsLoaded {
                translation_id,
                sections,
                needs_update,
                open_first_chapter,
            } => {
                if self.current_translation.as_ref().map(|t| t.id.as_str())
                    == Some(translation_id.as_str())
                    || self.current_translation.is_none()
                {
                    self.section_update_required = needs_update;
                    let _ = sender.output(ReaderWorkerOutput::SectionUpdateRequiredChanged(
                        needs_update,
                    ));

                    self.current_sections = sections.clone();
                    let _ = sender.output(ReaderWorkerOutput::SectionsUpdated(sections.clone()));

                    if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
                        tab.sections = sections.clone();
                    }
                    self.emit_tabs(&sender);

                    if open_first_chapter
                        && let Some(first_book) =
                            sections.iter().flat_map(|s| s.books.iter()).next()
                    {
                        sender.input(ReaderWorkerInput::OpenChapter {
                            book_id: first_book.id.clone(),
                            chapter: 1,
                        });
                    }
                }
            }

            ReaderWorkerInput::SelectTranslation(translation) => {
                self.current_translation = Some(translation.clone());
                if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
                    tab.translation = Some(translation.clone());
                    tab.commentary_id = None;
                    tab.commentary_name = String::new();
                    tab.book_id = None;
                    tab.chapter_data = None;
                    tab.commentary_data = None;
                }
                self.request_sections(translation.id, true, &sender);
            }

            ReaderWorkerInput::SelectParallelTranslation(parallel) => {
                self.parallel_translation = parallel.clone();
                if let Some(current) = self.current_chapter_data.clone() {
                    if let Some(p_trans) = parallel {
                        let reader = Arc::clone(&self.reader);
                        let translation = self
                            .current_translation
                            .clone()
                            .unwrap_or_else(|| p_trans.clone());
                        let book_id = current.book_id.clone();
                        let chapter_num = current.chapter_number;
                        let tx = sender.input_sender().clone();

                        std::thread::spawn(move || {
                            let p_data = reader.read_chapter(
                                p_trans.id.clone(),
                                book_id.clone(),
                                chapter_num,
                            );
                            let _ = tx.send(ReaderWorkerInput::ChapterLoaded {
                                translation,
                                book_id,
                                chapter: chapter_num,
                                data: Some(current),
                                parallel_data: p_data,
                            });
                        });
                    } else {
                        self.parallel_chapter_data = None;
                        let _ = sender.output(ReaderWorkerOutput::ChapterChanged {
                            chapter: self.current_chapter_data.clone(),
                            parallel: None,
                            translation: self.current_translation.clone(),
                        });
                    }
                }
            }

            ReaderWorkerInput::OpenChapter { book_id, chapter } => {
                let Some(translation) = self
                    .tabs
                    .get(self.active_tab_index)
                    .and_then(|t| t.translation.clone())
                    .or_else(|| self.current_translation.clone())
                else {
                    let _ = sender.output(ReaderWorkerOutput::Error(
                        "No translation selected".to_string(),
                    ));
                    return;
                };

                self.set_loading_content(true, &sender);
                let reader = Arc::clone(&self.reader);
                let parallel = self
                    .parallel_translation
                    .clone()
                    .filter(|p| p.id != translation.id);
                let tx = sender.input_sender().clone();

                std::thread::spawn(move || {
                    let data =
                        reader.read_chapter(translation.id.clone(), book_id.clone(), chapter);
                    let parallel_data = parallel
                        .and_then(|p| reader.read_chapter(p.id.clone(), book_id.clone(), chapter));
                    let _ = tx.send(ReaderWorkerInput::ChapterLoaded {
                        translation,
                        book_id,
                        chapter,
                        data,
                        parallel_data,
                    });
                });
            }

            ReaderWorkerInput::ChapterLoaded {
                translation,
                book_id,
                chapter,
                data,
                parallel_data,
            } => {
                self.set_loading_content(false, &sender);

                let Some(chapter_data) = data else {
                    let _ =
                        sender.output(ReaderWorkerOutput::Error("Chapter not found".to_string()));
                    return;
                };

                self.current_translation = Some(translation.clone());
                self.current_chapter_data = Some(chapter_data.clone());
                self.parallel_chapter_data = parallel_data.clone();

                self.current_commentary_id = None;
                self.current_commentary_name.clear();
                self.current_commentary_data = None;

                if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
                    tab.translation = Some(translation.clone());
                    tab.commentary_id = None;
                    tab.commentary_name.clear();
                    tab.book_id = Some(book_id);
                    tab.book_name = chapter_data.book_name.clone();
                    tab.chapter = chapter;
                    tab.chapter_data = Some(chapter_data.clone());
                    tab.commentary_data = None;
                }

                let _ = sender.output(ReaderWorkerOutput::ChapterChanged {
                    chapter: Some(chapter_data.clone()),
                    parallel: parallel_data,
                    translation: Some(translation),
                });
                let _ = sender.output(ReaderWorkerOutput::CommentaryChanged {
                    commentary_id: None,
                    commentary_name: String::new(),
                    data: None,
                });

                self.emit_tabs(&sender);
                self.request_chapter_entities(&chapter_data, &sender);
                sender.input(ReaderWorkerInput::RefreshStudyCommentary);
            }

            ReaderWorkerInput::SelectBook(book_id) => {
                if let Some(tab) = self.tabs.get(self.active_tab_index)
                    && tab.is_commentary()
                    && let Some(cid) = tab.commentary_id.clone()
                {
                    sender.input(ReaderWorkerInput::OpenCommentaryChapter {
                        commentary_id: cid,
                        commentary_name: tab.commentary_name.clone(),
                        book_id,
                        chapter: 1,
                    });
                    return;
                }

                sender.input(ReaderWorkerInput::OpenChapter {
                    book_id,
                    chapter: 1,
                });
            }

            ReaderWorkerInput::NextChapter => {
                let Some(tab) = self.tabs.get(self.active_tab_index).cloned() else {
                    return;
                };

                if tab.is_commentary() {
                    let (Some(cid), Some(data)) =
                        (tab.commentary_id.as_ref(), tab.commentary_data.as_ref())
                    else {
                        return;
                    };
                    let cid = cid.clone();
                    let books = tab.books();
                    if let Some(next) = data.next_chapter {
                        sender.input(ReaderWorkerInput::OpenCommentaryChapter {
                            commentary_id: cid,
                            commentary_name: tab.commentary_name,
                            book_id: data.book_id.clone(),
                            chapter: next,
                        });
                        return;
                    }
                    if let Some(book) = books.iter().find(|b| b.id == data.book_id)
                        && data.chapter_number < book.number_of_chapters
                    {
                        sender.input(ReaderWorkerInput::OpenCommentaryChapter {
                            commentary_id: cid,
                            commentary_name: tab.commentary_name,
                            book_id: data.book_id.clone(),
                            chapter: data.chapter_number + 1,
                        });
                        return;
                    }
                    if let Some(idx) = books.iter().position(|b| b.id == data.book_id)
                        && let Some(next_book) = books.get(idx + 1)
                    {
                        sender.input(ReaderWorkerInput::OpenCommentaryChapter {
                            commentary_id: cid,
                            commentary_name: tab.commentary_name,
                            book_id: next_book.id.clone(),
                            chapter: 1,
                        });
                    }
                    return;
                }

                let Some(data) = tab
                    .chapter_data
                    .as_ref()
                    .or(self.current_chapter_data.as_ref())
                else {
                    return;
                };

                if let Some(next) = data.next_chapter {
                    sender.input(ReaderWorkerInput::OpenChapter {
                        book_id: data.book_id.clone(),
                        chapter: next,
                    });
                    return;
                }

                let books = if tab.sections.is_empty() {
                    self.current_sections
                        .iter()
                        .flat_map(|s| s.books.iter().cloned())
                        .collect()
                } else {
                    tab.books()
                };

                if let Some(idx) = books.iter().position(|b| b.id == data.book_id)
                    && let Some(next_book) = books.get(idx + 1)
                {
                    sender.input(ReaderWorkerInput::OpenChapter {
                        book_id: next_book.id.clone(),
                        chapter: 1,
                    });
                }
            }

            ReaderWorkerInput::PreviousChapter => {
                let Some(tab) = self.tabs.get(self.active_tab_index).cloned() else {
                    return;
                };

                if tab.is_commentary() {
                    let (Some(cid), Some(data)) =
                        (tab.commentary_id.as_ref(), tab.commentary_data.as_ref())
                    else {
                        return;
                    };
                    let cid = cid.clone();
                    let books = tab.books();
                    if let Some(prev) = data.prev_chapter {
                        sender.input(ReaderWorkerInput::OpenCommentaryChapter {
                            commentary_id: cid,
                            commentary_name: tab.commentary_name,
                            book_id: data.book_id.clone(),
                            chapter: prev,
                        });
                        return;
                    }
                    if data.chapter_number > 1 {
                        sender.input(ReaderWorkerInput::OpenCommentaryChapter {
                            commentary_id: cid,
                            commentary_name: tab.commentary_name,
                            book_id: data.book_id.clone(),
                            chapter: data.chapter_number - 1,
                        });
                        return;
                    }
                    if let Some(idx) = books.iter().position(|b| b.id == data.book_id)
                        && idx > 0
                        && let Some(prev_book) = books.get(idx - 1)
                    {
                        sender.input(ReaderWorkerInput::OpenCommentaryChapter {
                            commentary_id: cid,
                            commentary_name: tab.commentary_name,
                            book_id: prev_book.id.clone(),
                            chapter: prev_book.number_of_chapters,
                        });
                    }
                    return;
                }

                let Some(data) = tab
                    .chapter_data
                    .as_ref()
                    .or(self.current_chapter_data.as_ref())
                else {
                    return;
                };

                if let Some(prev) = data.prev_chapter {
                    sender.input(ReaderWorkerInput::OpenChapter {
                        book_id: data.book_id.clone(),
                        chapter: prev,
                    });
                    return;
                }

                let books = if tab.sections.is_empty() {
                    self.current_sections
                        .iter()
                        .flat_map(|s| s.books.iter().cloned())
                        .collect()
                } else {
                    tab.books()
                };

                if let Some(idx) = books.iter().position(|b| b.id == data.book_id)
                    && idx > 0
                    && let Some(prev_book) = books.get(idx - 1)
                {
                    sender.input(ReaderWorkerInput::OpenChapter {
                        book_id: prev_book.id.clone(),
                        chapter: prev_book.number_of_chapters,
                    });
                }
            }

            ReaderWorkerInput::SetSelectedCanon(canon) => {
                let Some(translation) = self.current_translation.clone() else {
                    let _ = sender.output(ReaderWorkerOutput::Error(
                        "No translation selected".to_string(),
                    ));
                    return;
                };

                let reader = Arc::clone(&self.reader);
                let tx = sender.input_sender().clone();
                std::thread::spawn(move || {
                    if let Err(err) = reader.set_selected_canon(translation.id.clone(), canon) {
                        warn!("[ReaderWorker] set_selected_canon failed: {err}");
                    }
                    let sections = reader.get_books(translation.id.clone());
                    let needs_update = reader
                        .needs_section_update(translation.id.clone())
                        .unwrap_or(false);
                    let _ = tx.send(ReaderWorkerInput::SectionsLoaded {
                        translation_id: translation.id,
                        sections,
                        needs_update,
                        open_first_chapter: false,
                    });
                });
            }

            ReaderWorkerInput::ChapterEntitiesLoaded {
                book_id,
                chapter,
                entities,
            } => {
                let matches_current = self
                    .current_chapter_data
                    .as_ref()
                    .is_some_and(|c| c.book_id == book_id && c.chapter_number == chapter);
                if matches_current {
                    self.chapter_entities = entities.clone();
                    let _ = sender.output(ReaderWorkerOutput::ChapterEntitiesUpdated(entities));
                }
            }

            ReaderWorkerInput::LoadEntitiesForVerse {
                book_id,
                chapter,
                verse,
            } => {
                let reader = Arc::clone(&self.reader);
                let tx = sender.input_sender().clone();
                std::thread::spawn(move || {
                    let refs = reader.get_entities_for_verse(
                        THEOGRAPHIC_DATASET.to_string(),
                        book_id,
                        chapter,
                        verse,
                    );
                    let _ = tx.send(ReaderWorkerInput::VerseEntitiesLoaded(refs));
                });
            }

            ReaderWorkerInput::VerseEntitiesLoaded(refs) => {
                self.verse_entities = refs.clone();
                let _ = sender.output(ReaderWorkerOutput::VerseEntitiesUpdated(refs));
            }

            ReaderWorkerInput::OpenCommentaryChapter {
                commentary_id,
                commentary_name,
                book_id,
                chapter,
            } => {
                self.set_loading_content(true, &sender);
                let reader = Arc::clone(&self.reader);
                let tx = sender.input_sender().clone();

                std::thread::spawn(move || {
                    let result = reader
                        .read_commentary_chapter(commentary_id.clone(), book_id.clone(), chapter)
                        .map_err(|e| e.to_string());
                    let _ = tx.send(ReaderWorkerInput::CommentaryChapterLoaded {
                        commentary_id,
                        commentary_name,
                        book_id,
                        chapter,
                        result,
                    });
                });
            }

            ReaderWorkerInput::CommentaryChapterLoaded {
                commentary_id,
                commentary_name,
                book_id,
                chapter,
                result,
            } => {
                self.set_loading_content(false, &sender);

                match result {
                    Ok(Some(data)) => {
                        self.current_commentary_id = Some(commentary_id.clone());
                        self.current_commentary_name = commentary_name.clone();
                        self.current_commentary_data = Some(data.clone());

                        self.current_translation = None;
                        self.current_chapter_data = None;
                        self.parallel_chapter_data = None;

                        if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
                            tab.translation = None;
                            tab.commentary_id = Some(commentary_id.clone());
                            tab.commentary_name = commentary_name.clone();
                            tab.book_id = Some(book_id);
                            tab.book_name = data.book_name.clone();
                            tab.chapter = chapter;
                            tab.chapter_data = None;
                            tab.commentary_data = Some(data.clone());
                        }

                        let _ = sender.output(ReaderWorkerOutput::CommentaryChanged {
                            commentary_id: Some(commentary_id),
                            commentary_name,
                            data: Some(data),
                        });
                        let _ = sender.output(ReaderWorkerOutput::ChapterChanged {
                            chapter: None,
                            parallel: None,
                            translation: None,
                        });
                        self.emit_tabs(&sender);
                    }
                    Ok(None) => {
                        let _ = sender.output(ReaderWorkerOutput::Error(format!(
                            "No commentary for {book_id} {chapter}"
                        )));
                    }
                    Err(err) => {
                        let _ = sender.output(ReaderWorkerOutput::Error(format!(
                            "Failed to load commentary: {err}"
                        )));
                    }
                }
            }

            ReaderWorkerInput::OpenCommentaryForCurrentChapter {
                commentary_id,
                commentary_name,
            } => {
                let Some(data) = self.current_chapter_data.clone() else {
                    let _ = sender.output(ReaderWorkerOutput::Error(
                        "No chapter loaded to cross-reference".to_string(),
                    ));
                    return;
                };

                self.next_tab_seq += 1;
                let mut tab =
                    ReaderTabState::new_read_tab(format!("tab-{}", self.next_tab_seq), None);
                tab.commentary_id = Some(commentary_id.clone());
                tab.commentary_name = commentary_name.clone();
                tab.sections = self.current_sections.clone();
                tab.book_id = Some(data.book_id.clone());
                tab.book_name = data.book_name.clone();
                tab.chapter = data.chapter_number;

                self.tabs.push(tab);
                self.active_tab_index = self.tabs.len().saturating_sub(1);
                self.emit_tabs(&sender);

                sender.input(ReaderWorkerInput::OpenCommentaryChapter {
                    commentary_id,
                    commentary_name,
                    book_id: data.book_id,
                    chapter: data.chapter_number,
                });
            }

            ReaderWorkerInput::SetStudyCommentary { id, name } => {
                self.study_commentary_id = Some(id);
                self.study_commentary_name = name;
                sender.input(ReaderWorkerInput::RefreshStudyCommentary);
            }

            ReaderWorkerInput::ClearStudyCommentary => {
                self.study_commentary_id = None;
                self.study_commentary_name.clear();
                self.study_commentary_data = None;
                self.study_commentary_loading = false;
                let _ = sender.output(ReaderWorkerOutput::StudyCommentaryUpdated {
                    id: None,
                    name: String::new(),
                    data: None,
                    loading: false,
                });
            }

            ReaderWorkerInput::RefreshStudyCommentary => {
                let Some(id) = self.study_commentary_id.clone() else {
                    return;
                };
                let Some(data) = self.current_chapter_data.clone() else {
                    self.study_commentary_data = None;
                    let _ = sender.output(ReaderWorkerOutput::StudyCommentaryUpdated {
                        id: Some(id),
                        name: self.study_commentary_name.clone(),
                        data: None,
                        loading: false,
                    });
                    return;
                };

                self.study_commentary_loading = true;
                let _ = sender.output(ReaderWorkerOutput::StudyCommentaryUpdated {
                    id: Some(id.clone()),
                    name: self.study_commentary_name.clone(),
                    data: self.study_commentary_data.clone(),
                    loading: true,
                });

                let reader = Arc::clone(&self.reader);
                let tx = sender.input_sender().clone();
                std::thread::spawn(move || {
                    let res = reader
                        .read_commentary_chapter(id, data.book_id, data.chapter_number)
                        .map_err(|e| e.to_string());
                    let _ = tx.send(ReaderWorkerInput::StudyCommentaryLoaded(res));
                });
            }

            ReaderWorkerInput::StudyCommentaryLoaded(res) => {
                self.study_commentary_loading = false;
                match res {
                    Ok(data) => {
                        self.study_commentary_data = data.clone();
                        let _ = sender.output(ReaderWorkerOutput::StudyCommentaryUpdated {
                            id: self.study_commentary_id.clone(),
                            name: self.study_commentary_name.clone(),
                            data,
                            loading: false,
                        });
                    }
                    Err(err) => {
                        warn!("[ReaderWorker] study commentary failed: {err}");
                        self.study_commentary_data = None;
                        let _ = sender.output(ReaderWorkerOutput::StudyCommentaryUpdated {
                            id: self.study_commentary_id.clone(),
                            name: self.study_commentary_name.clone(),
                            data: None,
                            loading: false,
                        });
                    }
                }
            }

            ReaderWorkerInput::SwitchTab(index) => {
                if index >= self.tabs.len() {
                    return;
                }
                self.active_tab_index = index;
                self.apply_active_tab_state(&sender);
                self.emit_tabs(&sender);

                if let Some(tab) = self.tabs.get(index).cloned() {
                    if tab.kind == ReaderTabKind::Search {
                        if !tab.search_query.trim().is_empty() {
                            sender.input(ReaderWorkerInput::PerformSearch(tab.search_query));
                        }
                    } else if tab.is_commentary() {
                        if let (Some(cid), Some(book_id)) = (tab.commentary_id, tab.book_id) {
                            sender.input(ReaderWorkerInput::OpenCommentaryChapter {
                                commentary_id: cid,
                                commentary_name: tab.commentary_name,
                                book_id,
                                chapter: tab.chapter,
                            });
                        }
                    } else if let Some(translation) = tab.translation {
                        let book_id = tab.book_id;
                        let chapter = tab.chapter;
                        self.current_translation = Some(translation.clone());
                        if let Some(bid) = book_id {
                            sender.input(ReaderWorkerInput::OpenChapter {
                                book_id: bid,
                                chapter,
                            });
                        } else {
                            self.request_sections(translation.id, true, &sender);
                        }
                    }
                }
            }

            ReaderWorkerInput::AddNewTab => {
                let Some(translation) = self
                    .current_translation
                    .clone()
                    .or_else(|| self.installed_translations.first().cloned())
                else {
                    return;
                };

                self.next_tab_seq += 1;
                let tab = ReaderTabState::new_read_tab(
                    format!("tab-{}", self.next_tab_seq),
                    Some(translation.clone()),
                );
                self.tabs.push(tab);
                self.active_tab_index = self.tabs.len().saturating_sub(1);
                self.current_translation = Some(translation.clone());
                self.emit_tabs(&sender);
                self.request_sections(translation.id, true, &sender);
            }

            ReaderWorkerInput::AddSearchTab => {
                self.next_tab_seq += 1;
                let tab = ReaderTabState::new_search_tab(
                    format!("tab-{}", self.next_tab_seq),
                    self.current_translation.clone(),
                );
                self.tabs.push(tab);
                self.active_tab_index = self.tabs.len().saturating_sub(1);
                self.emit_tabs(&sender);
            }

            ReaderWorkerInput::RemoveTab(index) => {
                if self.tabs.len() <= 1 || index >= self.tabs.len() {
                    return;
                }
                self.tabs.remove(index);
                if index < self.active_tab_index {
                    self.active_tab_index = self.active_tab_index.saturating_sub(1);
                } else if index == self.active_tab_index {
                    self.active_tab_index = self.active_tab_index.min(self.tabs.len() - 1);
                }
                self.apply_active_tab_state(&sender);
                self.emit_tabs(&sender);
            }

            ReaderWorkerInput::SetSearchQuery(query) => {
                if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
                    tab.search_query = query;
                }
                self.emit_tabs(&sender);
            }

            ReaderWorkerInput::SetSearchType(search_type) => {
                let query_to_run = if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
                    tab.search_type = search_type;
                    tab.search_query.clone()
                } else {
                    String::new()
                };
                self.emit_tabs(&sender);
                if !query_to_run.trim().is_empty() {
                    sender.input(ReaderWorkerInput::PerformSearch(query_to_run));
                }
            }

            ReaderWorkerInput::ToggleSearchSection(section) => {
                let query_to_run = if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
                    if tab.selected_sections.contains(&section) {
                        if tab.selected_sections.len() > 1 {
                            tab.selected_sections.remove(&section);
                        }
                    } else {
                        tab.selected_sections.insert(section);
                    }
                    tab.search_query.clone()
                } else {
                    String::new()
                };
                self.emit_tabs(&sender);
                if !query_to_run.trim().is_empty() {
                    sender.input(ReaderWorkerInput::PerformSearch(query_to_run));
                }
            }

            ReaderWorkerInput::ClearSearch => {
                self.search_generation += 1;
                self.search_phase = SearchPhase::Done;
                self.search_fraction = 0.0;
                self.search_message.clear();

                if let Some(tab) = self.tabs.get_mut(self.active_tab_index) {
                    tab.is_searching = false;
                    tab.search_results = None;
                    tab.search_error = None;
                }

                let _ = sender.output(ReaderWorkerOutput::SearchProgress {
                    phase: SearchPhase::Done,
                    fraction: 0.0,
                    message: String::new(),
                });
                let _ = sender.output(ReaderWorkerOutput::SearchResultsUpdated {
                    tab_index: self.active_tab_index,
                    is_searching: false,
                    results: None,
                    error: None,
                });
            }

            ReaderWorkerInput::PerformSearch(query) => {
                let trimmed = query.trim().to_string();
                if trimmed.is_empty() {
                    sender.input(ReaderWorkerInput::ClearSearch);
                    return;
                }

                let tab_index = self.active_tab_index;
                let Some(tab) = self.tabs.get_mut(tab_index) else {
                    return;
                };

                self.search_generation += 1;
                let generation = self.search_generation;
                self.search_phase = SearchPhase::Classifying;
                self.search_fraction = 0.0;
                self.search_message.clear();

                tab.search_query = query;
                tab.is_searching = true;
                tab.search_error = None;

                let translation_id = tab
                    .translation
                    .as_ref()
                    .map(|t| t.id.clone())
                    .or_else(|| self.current_translation.as_ref().map(|t| t.id.clone()));

                let scoped_book_ids: Vec<String> =
                    if tab.selected_sections == CanonSection::all_set() {
                        Vec::new()
                    } else {
                        tab.sections
                            .iter()
                            .filter(|sec| {
                                tab.selected_sections
                                    .contains(&CanonSection::from_wire(&sec.section_key))
                            })
                            .flat_map(|sec| sec.books.iter().map(|b| b.id.clone()))
                            .collect()
                    };

                let filter = SearchFilter {
                    book_ids: scoped_book_ids,
                    mode: tab.search_type.engine_mode(),
                };

                let _ = sender.output(ReaderWorkerOutput::SearchProgress {
                    phase: SearchPhase::Classifying,
                    fraction: 0.0,
                    message: String::new(),
                });
                let _ = sender.output(ReaderWorkerOutput::SearchResultsUpdated {
                    tab_index,
                    is_searching: true,
                    results: None,
                    error: None,
                });

                let reader = Arc::clone(&self.reader);
                let tx = sender.input_sender().clone();
                std::thread::spawn(move || {
                    let res = reader
                        .search(trimmed, translation_id, DEFAULT_SEARCH_LIMIT, filter, None)
                        .map_err(|e| e.to_string());
                    let _ = tx.send(ReaderWorkerInput::SearchCompleted {
                        generation,
                        tab_index,
                        result: res,
                    });
                });
            }

            ReaderWorkerInput::SearchCompleted {
                generation,
                tab_index,
                result,
            } => {
                if generation != self.search_generation {
                    return;
                }

                self.search_phase = SearchPhase::Done;
                self.search_fraction = 1.0;

                match result {
                    Ok(results) => {
                        if let Some(tab) = self.tabs.get_mut(tab_index) {
                            tab.is_searching = false;
                            tab.search_results = Some(results.clone());
                            tab.search_error = None;
                        }
                        let _ = sender.output(ReaderWorkerOutput::SearchResultsUpdated {
                            tab_index,
                            is_searching: false,
                            results: Some(results),
                            error: None,
                        });
                    }
                    Err(err) => {
                        if let Some(tab) = self.tabs.get_mut(tab_index) {
                            tab.is_searching = false;
                            tab.search_error = Some(err.clone());
                        }
                        let _ = sender.output(ReaderWorkerOutput::SearchResultsUpdated {
                            tab_index,
                            is_searching: false,
                            results: None,
                            error: Some(err),
                        });
                    }
                }
            }
        }
    }
}

impl ReaderWorker {
    fn set_loading_content(&mut self, loading: bool, sender: &ComponentSender<Self>) {
        self.is_loading_content = loading;
        let _ = sender.output(ReaderWorkerOutput::LoadingContentChanged(loading));
    }

    fn emit_tabs(&self, sender: &ComponentSender<Self>) {
        let _ = sender.output(ReaderWorkerOutput::TabsUpdated {
            tabs: self.tabs.clone(),
            active_index: self.active_tab_index,
        });
    }

    fn apply_active_tab_state(&mut self, sender: &ComponentSender<Self>) {
        let Some(tab) = self.tabs.get(self.active_tab_index).cloned() else {
            return;
        };
        self.current_translation = tab.translation;
        self.current_commentary_id = tab.commentary_id;
        self.current_commentary_name = tab.commentary_name;
        self.current_sections = tab.sections;
        self.current_chapter_data = tab.chapter_data;
        self.current_commentary_data = tab.commentary_data;

        let _ = sender.output(ReaderWorkerOutput::ChapterChanged {
            chapter: self.current_chapter_data.clone(),
            parallel: self.parallel_chapter_data.clone(),
            translation: self.current_translation.clone(),
        });
        let _ = sender.output(ReaderWorkerOutput::CommentaryChanged {
            commentary_id: self.current_commentary_id.clone(),
            commentary_name: self.current_commentary_name.clone(),
            data: self.current_commentary_data.clone(),
        });
    }

    fn request_sections(
        &self,
        translation_id: String,
        open_first_chapter: bool,
        sender: &ComponentSender<Self>,
    ) {
        let reader = Arc::clone(&self.reader);
        let tx = sender.input_sender().clone();
        std::thread::spawn(move || {
            let sections = reader.get_books(translation_id.clone());
            let needs_update = reader
                .needs_section_update(translation_id.clone())
                .unwrap_or(false);
            let _ = tx.send(ReaderWorkerInput::SectionsLoaded {
                translation_id,
                sections,
                needs_update,
                open_first_chapter,
            });
        });
    }

    fn request_chapter_entities(&self, chapter: &ReaderChapter, sender: &ComponentSender<Self>) {
        let mut verse_set = HashSet::new();
        for block in &chapter.blocks {
            for verse in &block.verses {
                verse_set.insert(verse.number);
            }
        }

        if verse_set.is_empty() {
            let _ = sender.output(ReaderWorkerOutput::ChapterEntitiesUpdated(Vec::new()));
            return;
        }

        let verse_numbers: Vec<i32> = verse_set.into_iter().collect();
        let book_id = chapter.book_id.clone();
        let chapter_num = chapter.chapter_number;
        let reader = Arc::clone(&self.reader);
        let tx = sender.input_sender().clone();

        std::thread::spawn(move || {
            let mut seen: HashMap<String, EntityRef> = HashMap::new();
            let mut order: Vec<String> = Vec::new();

            for v in verse_numbers {
                let refs = reader.get_entities_for_verse(
                    THEOGRAPHIC_DATASET.to_string(),
                    book_id.clone(),
                    chapter_num,
                    v,
                );
                for entity in refs {
                    let key = format!("{}:{}", entity.kind, entity.id);
                    if !seen.contains_key(&key) {
                        seen.insert(key.clone(), entity);
                        order.push(key);
                    }
                }
            }

            let deduped: Vec<EntityRef> =
                order.into_iter().filter_map(|k| seen.remove(&k)).collect();
            let _ = tx.send(ReaderWorkerInput::ChapterEntitiesLoaded {
                book_id,
                chapter: chapter_num,
                entities: deduped,
            });
        });
    }
}
