use std::collections::HashSet;

use adw::prelude::*;
use gettextrs::gettext;
use relm4::{WorkerController, prelude::*};
use xbible_engine::features::reader::{
    BookSection, CommentaryChapter, EntityRef, ReaderChapter, search::SearchResults,
};
use xbible_engine::features::store::StoreItem;
use xbible_engine::sources::models::{GenericBook, GenericTranslation};

use crate::{
    core::{
        engine::SharedEngine,
        ui::{
            highlights::{HighlightStore, HighlightStyle, PersistentHighlight},
            text_config::TextConfig,
        },
    },
    features::study::study_comonents::{
        chapter_renderer::{
            ChapterViewEvent, StudyToolKind, VerseContextAction, build_chapter_view,
        },
        commentary_panel::build_commentary_panel,
        page_control_button::ExpandingThemeMenu,
        search_panel::{SearchPanelEvent, build_search_panel},
    },
    workers::reader_worker::{
        CanonSection, ReaderTabKind, ReaderWorker, ReaderWorkerInput, ReaderWorkerOutput,
        SearchType,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StudyPanelMode {
    Read,
    Commentary,
    Search,
}

#[derive(Debug, Clone)]
pub struct HeaderState {
    pub module: Option<GenericTranslation>,
    pub book: Option<GenericBook>,
    pub chapter: Option<i32>,
}

#[derive(Debug)]
struct TabSlot {
    page: adw::TabPage,
    stack: gtk::Stack,
    mode: StudyPanelMode,
    read_overlay: gtk::Overlay,
    read_scroll: gtk::ScrolledWindow,
    book_filter: Option<String>,
}

#[derive(Debug)]
pub(crate) struct StudyView {
    tab_counter: usize,
    tabs: Vec<TabSlot>,
    text_config: TextConfig,
    highlights: HighlightStore,
    current_chapter: Option<ReaderChapter>,
    chapter_entities: Vec<EntityRef>,
    selected_verse: Option<i32>,
    is_loading_content: bool,
    commentaries: Vec<StoreItem>,
    study_commentary_id: Option<String>,
    study_commentary_name: String,
    study_commentary_data: Option<CommentaryChapter>,
    study_commentary_loading: bool,
    sections: Vec<BookSection>,
    search_query: String,
    search_type: SearchType,
    selected_sections: HashSet<CanonSection>,
    is_searching: bool,
    search_progress_fraction: f64,
    search_progress_message: String,
    search_results: Option<SearchResults>,
    search_error: Option<String>,
    reader_worker: WorkerController<ReaderWorker>,
    installed_translations: Vec<GenericTranslation>,
    active_header_state: Option<HeaderState>,
    selected_translation_id: Option<String>,
    selected_book_id: Option<String>,
    selected_chapter: Option<i32>,
    page_controls: Controller<ExpandingThemeMenu>,
}

#[derive(Debug)]
pub(crate) enum StudyViewInput {
    NewTab,
    OpenSearch,
    OpenCommentary,
    TabSelected(adw::TabPage),
    CloseTab(adw::TabPage),
    SetTextConfig(TextConfig),
    SetHighlights(HighlightStore),
    SetChapter {
        chapter: Option<ReaderChapter>,
        entities: Vec<EntityRef>,
        loading: bool,
    },
    SetCommentaries {
        commentaries: Vec<StoreItem>,
        selected_id: Option<String>,
        name: String,
        chapter: Option<CommentaryChapter>,
        loading: bool,
    },
    SetSearchState {
        sections: Vec<BookSection>,
        query: String,
        search_type: SearchType,
        selected_sections: HashSet<CanonSection>,
        is_searching: bool,
        progress_fraction: f64,
        progress_message: String,
        results: Option<SearchResults>,
        error: Option<String>,
    },
    ChapterEvent(ChapterViewEvent),
    SearchEvent(SearchPanelEvent),
    CommentarySelected(String, String),
    ReaderWorkerEvent(ReaderWorkerOutput),
    SelectTranslation(GenericTranslation),
    SelectBook(String),
    SelectChapter(i32),
    OpenStore,
    RefreshVersions,
    PreviousChapter,
    NextChapter,
    OpenChapterPicker,
    OpenThemePopup,
}

#[derive(Debug, Clone)]
pub(crate) enum StudyViewOutput {
    ShowError(String),
    ToggleSidebar,
    OpenChapterRequested {
        book_id: String,
        chapter: i32,
    },
    SelectCommentaryRequested {
        id: String,
        name: String,
    },
    SearchQueryChanged(String),
    ClearSearchRequested,
    SearchTypeChanged(SearchType),
    ResetSearchSectionsRequested,
    ToggleSearchSectionRequested(CanonSection),
    HighlightChanged(HighlightStore),
    CopyVerseRequested {
        book_id: String,
        chapter: i32,
        verse: i32,
    },
    OpenStoreRequested,
}

#[relm4::component(pub)]
impl relm4::Component for StudyView {
    type Init = SharedEngine;
    type Input = StudyViewInput;
    type Output = StudyViewOutput;
    type CommandOutput = ();

    view! {
        adw::NavigationPage {
            set_title: &gettext("Study"),
            #[wrap(Some)]
            set_child = &adw::TabOverview {
                #[name = "tab_overview"]
                #[wrap(Some)]
                set_child = &adw::ToolbarView {
                    add_top_bar = &adw::HeaderBar {
                        pack_start = &gtk::Button {
                            set_icon_name: "sidebar-show-symbolic",
                            set_valign: gtk::Align::Center,
                            set_tooltip_text: Some(&gettext("Toggle Sidebar")),
                            connect_clicked[sender] => move |_| {
                                let _ = sender.output(StudyViewOutput::ToggleSidebar);
                            },
                        },
                        #[wrap(Some)]
                        set_title_widget = &gtk::Box {
                            add_css_class: "linked",
                            add_css_class: "rounded",
                            set_halign: gtk::Align::Center,
                            set_margin_bottom: 10,
                            gtk::MenuButton {
                                set_tooltip_text: Some("Select Version"),
                                #[wrap(Some)]
                                set_child = &gtk::Box {
                                    set_spacing: 4,
                                    gtk::Label {
                                        #[watch]
                                        set_label: model.active_header_state.as_ref()
                                            .and_then(|s| s.module.as_ref())
                                            .map(|m| m.name.as_str())
                                            .unwrap_or("Select Version"),
                                    },
                                    gtk::Image { set_icon_name: Some("pan-down-symbolic") },
                                },
                                #[wrap(Some)]
                                set_popover = &gtk::Popover {
                                    set_autohide: true,
                                    connect_show[sender] => move |_| {
                                        sender.input(StudyViewInput::RefreshVersions);
                                    },
                                    gtk::ScrolledWindow {
                                        set_hscrollbar_policy: gtk::PolicyType::Never,
                                        set_min_content_width: 320,
                                        set_max_content_height: 500,
                                        #[name = "version_list"]
                                        gtk::Box {
                                            set_orientation: gtk::Orientation::Vertical,
                                            set_spacing: 4,
                                            set_margin_all: 8,
                                        },
                                    },
                                },
                            },
                            gtk::MenuButton {
                                set_tooltip_text: Some("Select Book"),
                                #[wrap(Some)]
                                set_child = &gtk::Box {
                                    set_spacing: 4,
                                    gtk::Label {
                                        #[watch]
                                        set_label: model.active_header_state.as_ref()
                                            .and_then(|s| s.book.as_ref())
                                            .map(|b| b.name.as_str())
                                            .unwrap_or("Select Book"),
                                    },
                                    gtk::Image { set_icon_name: Some("pan-down-symbolic") },
                                },
                                #[wrap(Some)]
                                set_popover = &gtk::Popover {
                                    gtk::ScrolledWindow {
                                        set_hscrollbar_policy: gtk::PolicyType::Never,
                                        set_min_content_width: 420,
                                        set_max_content_height: 500,
                                        #[name = "book_list"]
                                        gtk::FlowBox {
                                            set_max_children_per_line: 4,
                                            set_min_children_per_line: 4,
                                            set_selection_mode: gtk::SelectionMode::None,
                                            set_column_spacing: 8,
                                            set_row_spacing: 8,
                                            set_margin_all: 8,
                                        },
                                    },
                                },
                            },
                            #[name = "chapter_menu"]
                            gtk::MenuButton {
                                set_tooltip_text: Some("Select Chapter"),
                                #[wrap(Some)]
                                set_child = &gtk::Box {
                                    set_spacing: 4,
                                    gtk::Label {
                                        #[watch]
                                        set_label: &model.active_header_state.as_ref()
                                            .and_then(|s| s.chapter)
                                            .map(|c| format!("Chapter {c}"))
                                            .unwrap_or_else(|| "Select Chapter".to_string()),
                                    },
                                    gtk::Image { set_icon_name: Some("pan-down-symbolic") },
                                },
                                #[wrap(Some)]
                                set_popover = &gtk::Popover {
                                    gtk::ScrolledWindow {
                                        set_hscrollbar_policy: gtk::PolicyType::Never,
                                        set_min_content_width: 240,
                                        set_max_content_height: 500,
                                        #[name = "chapter_list"]
                                        gtk::FlowBox {
                                            set_max_children_per_line: 5,
                                            set_min_children_per_line: 5,
                                            set_selection_mode: gtk::SelectionMode::None,
                                            set_column_spacing: 6,
                                            set_row_spacing: 6,
                                            set_margin_all: 8,
                                        },
                                    },
                                },
                            },
                        },
                        pack_end = &adw::TabButton {
                            set_view: Some(&tab_view),
                            set_valign: gtk::Align::Center,
                            set_action_name: Some("overview.open"),
                            set_tooltip_text: Some(&gettext("Open Tab Overview")),
                        },
                        pack_end = &gtk::Button {
                            set_icon_name: "tab-new-symbolic",
                            set_valign: gtk::Align::Center,
                            set_tooltip_text: Some(&gettext("New Tab")),
                            connect_clicked[sender] => move |_| sender.input(StudyViewInput::NewTab),
                        },
                        pack_end = &gtk::Button {
                            set_icon_name: "dictionary-symbolic",
                            set_valign: gtk::Align::Center,
                            set_tooltip_text: Some(&gettext("Commentary")),
                            connect_clicked[sender] => move |_| sender.input(StudyViewInput::OpenCommentary),
                        },
                        pack_end = &gtk::Button {
                            set_icon_name: "edit-find-symbolic",
                            set_valign: gtk::Align::Center,
                            set_tooltip_text: Some(&gettext("Open Search Tab")),
                            connect_clicked[sender] => move |_| sender.input(StudyViewInput::OpenSearch),
                        },
                    },
                    add_top_bar = &adw::TabBar {
                        set_autohide: false,
                        set_view: Some(&tab_view),
                    },
                    #[wrap(Some)]
                    #[name = "tab_view"]
                    set_content = &adw::TabView {
                        connect_selected_page_notify[sender] => move |view| {
                            if let Some(page) = view.selected_page() {
                                sender.input(StudyViewInput::TabSelected(page));
                            }
                        },
                        connect_close_page[sender] => move |_, page| {
                            sender.input(StudyViewInput::CloseTab(page.clone()));
                            glib::Propagation::Stop
                        },
                    },
                },
                set_view: Some(&tab_view),
            },
        }
    }

    fn init(
        engine: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let text_config = TextConfig::load().unwrap_or_else(|err| {
            warn!("Failed to load TextConfig, using defaults: {err}");
            TextConfig::default()
        });
        let highlights = HighlightStore::load().unwrap_or_else(|err| {
            warn!("Failed to load HighlightStore, using empty store: {err}");
            HighlightStore::default()
        });
        let reader_worker = ReaderWorker::builder()
            .detach_worker(engine.clone())
            .forward(sender.input_sender(), StudyViewInput::ReaderWorkerEvent);
        let page_controls = ExpandingThemeMenu::builder()
            .launch(TextConfig::load().unwrap_or_default())
            .forward(sender.input_sender(), |event| match event {
                crate::features::study::study_comonents::page_control_button::ThemeMenuOutput::PreviousChapter => StudyViewInput::PreviousChapter,
                crate::features::study::study_comonents::page_control_button::ThemeMenuOutput::NextChapter => StudyViewInput::NextChapter,
                crate::features::study::study_comonents::page_control_button::ThemeMenuOutput::OpenThemePopup => StudyViewInput::OpenThemePopup,
                crate::features::study::study_comonents::page_control_button::ThemeMenuOutput::DimBackground(_) => StudyViewInput::RefreshVersions,
            });

        let mut model = Self {
            tab_counter: 0,
            tabs: Vec::new(),
            text_config,
            highlights,
            current_chapter: None,
            chapter_entities: Vec::new(),
            selected_verse: None,
            is_loading_content: false,
            commentaries: Vec::new(),
            study_commentary_id: None,
            study_commentary_name: String::new(),
            study_commentary_data: None,
            study_commentary_loading: false,
            sections: Vec::new(),
            search_query: String::new(),
            search_type: SearchType::MultiWord,
            selected_sections: CanonSection::all_set(),
            is_searching: false,
            search_progress_fraction: 0.0,
            search_progress_message: String::new(),
            search_results: None,
            search_error: None,
            reader_worker,
            installed_translations: Vec::new(),
            active_header_state: None,
            selected_translation_id: None,
            selected_book_id: None,
            selected_chapter: None,
            page_controls,
        };
        let widgets = view_output!();

        let mut model = model;
        model.append_tab(
            &widgets.tab_view,
            &gettext("Reader"),
            StudyPanelMode::Read,
            false,
            &sender,
        );
        relm4::ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            StudyViewInput::NewTab => {
                self.tab_counter += 1;
                let title = format!("{} {}", gettext("Study Tab"), self.tab_counter);
                self.append_tab(
                    &widgets.tab_view,
                    &title,
                    StudyPanelMode::Read,
                    true,
                    &sender,
                );
            }
            StudyViewInput::OpenSearch => {
                self.tab_counter += 1;
                let title = format!("{} {}", gettext("Search"), self.tab_counter);
                self.append_tab(
                    &widgets.tab_view,
                    &title,
                    StudyPanelMode::Search,
                    true,
                    &sender,
                );
            }
            StudyViewInput::OpenCommentary => {
                self.tab_counter += 1;
                let title = format!("{} {}", gettext("Commentary"), self.tab_counter);
                self.append_tab(
                    &widgets.tab_view,
                    &title,
                    StudyPanelMode::Commentary,
                    true,
                    &sender,
                );
            }
            StudyViewInput::TabSelected(_) => self.refresh_all_tabs(&sender),
            StudyViewInput::CloseTab(page) => {
                widgets.tab_view.close_page_finish(&page, true);
                self.tabs.retain(|slot| slot.page != page);
            }
            StudyViewInput::SetTextConfig(config) => {
                self.text_config = config;
                self.refresh_all_tabs(&sender);
            }
            StudyViewInput::SetHighlights(highlights) => {
                self.highlights = highlights;
                self.refresh_all_tabs(&sender);
            }
            StudyViewInput::SetChapter {
                chapter,
                entities,
                loading,
            } => {
                self.current_chapter = chapter;
                self.chapter_entities = entities;
                self.is_loading_content = loading;
                self.sync_selection_from_chapter();
                self.update_header_state();
                self.refresh_all_tabs(&sender);
            }
            StudyViewInput::SetCommentaries {
                commentaries,
                selected_id,
                name,
                chapter,
                loading,
            } => {
                self.commentaries = commentaries;
                self.study_commentary_id = selected_id;
                self.study_commentary_name = name;
                self.study_commentary_data = chapter;
                self.study_commentary_loading = loading;
                self.refresh_all_tabs(&sender);
            }
            StudyViewInput::SetSearchState {
                sections,
                query,
                search_type,
                selected_sections,
                is_searching,
                progress_fraction,
                progress_message,
                results,
                error,
            } => {
                self.sections = sections;
                self.search_query = query;
                self.search_type = search_type;
                self.selected_sections = selected_sections;
                self.is_searching = is_searching;
                self.search_progress_fraction = progress_fraction;
                self.search_progress_message = progress_message;
                self.search_results = results;
                self.search_error = error;
                self.refresh_all_tabs(&sender);
            }
            StudyViewInput::ReaderWorkerEvent(event) => match event {
                ReaderWorkerOutput::InstalledTranslationsUpdated(translations) => {
                    let needs_initial_selection =
                        self.installed_translations.is_empty() && self.current_chapter.is_none();
                    self.installed_translations = translations;
                    if needs_initial_selection
                        && let Some(translation) = self.installed_translations.first().cloned()
                    {
                        self.selected_translation_id = Some(translation.id.clone());
                        self.update_header_state();
                        self.reader_worker
                            .emit(ReaderWorkerInput::SelectTranslation(translation));
                    }
                    self.refresh_version_list(&widgets.version_list, &sender);
                    self.update_header_state();
                    self.refresh_all_tabs(&sender);
                }
                ReaderWorkerOutput::SectionsUpdated(sections) => {
                    self.sections = sections;
                    let selected_book_is_valid = self.selected_book_id.as_ref().is_some_and(|id| {
                        self.sections
                            .iter()
                            .flat_map(|section| section.books.iter())
                            .any(|book| &book.id == id)
                    });
                    if !selected_book_is_valid
                        && let Some(book) = self
                            .sections
                            .iter()
                            .flat_map(|section| section.books.iter())
                            .next()
                    {
                        self.selected_book_id = Some(book.id.clone());
                        self.selected_chapter = Some(1);
                    }
                    self.refresh_book_list(&widgets.book_list, &sender);
                    self.refresh_chapter_list(&widgets.chapter_list, &sender);
                    self.update_header_state();
                    self.refresh_all_tabs(&sender);
                }
                ReaderWorkerOutput::ChapterChanged {
                    chapter,
                    translation,
                    ..
                } => {
                    self.current_chapter = chapter;
                    if let Some(chapter) = self.current_chapter.as_ref() {
                        self.selected_translation_id = Some(chapter.translation_id.clone());
                        self.selected_book_id = Some(chapter.book_id.clone());
                        self.selected_chapter = Some(chapter.chapter_number);
                    }
                    self.update_header_state_with_translation(translation);
                    self.refresh_chapter_list(&widgets.chapter_list, &sender);
                    self.refresh_all_tabs(&sender);
                }
                ReaderWorkerOutput::ChapterEntitiesUpdated(entities) => {
                    self.chapter_entities = entities;
                    self.refresh_all_tabs(&sender);
                }
                ReaderWorkerOutput::LoadingContentChanged(loading) => {
                    self.is_loading_content = loading;
                    self.refresh_all_tabs(&sender);
                }
                ReaderWorkerOutput::CommentariesUpdated(commentaries) => {
                    self.commentaries = commentaries;
                    self.refresh_all_tabs(&sender);
                }
                ReaderWorkerOutput::StudyCommentaryUpdated {
                    id,
                    name,
                    data,
                    loading,
                } => {
                    self.study_commentary_id = id;
                    self.study_commentary_name = name;
                    self.study_commentary_data = data;
                    self.study_commentary_loading = loading;
                    self.refresh_all_tabs(&sender);
                }
                ReaderWorkerOutput::SearchProgress {
                    fraction, message, ..
                } => {
                    self.search_progress_fraction = fraction;
                    self.search_progress_message = message;
                    self.refresh_all_tabs(&sender);
                }
                ReaderWorkerOutput::SearchResultsUpdated {
                    is_searching,
                    results,
                    error,
                    ..
                } => {
                    self.is_searching = is_searching;
                    self.search_results = results;
                    self.search_error = error;
                    self.refresh_all_tabs(&sender);
                }
                ReaderWorkerOutput::Error(error) => {
                    let _ = sender.output(StudyViewOutput::ShowError(error));
                }
                ReaderWorkerOutput::CommentaryChanged {
                    commentary_id,
                    commentary_name,
                    data,
                } => {
                    self.study_commentary_id = commentary_id;
                    self.study_commentary_name = commentary_name;
                    self.study_commentary_data = data;
                    self.study_commentary_loading = false;
                    self.refresh_all_tabs(&sender);
                }
                ReaderWorkerOutput::ReadyChanged(_)
                | ReaderWorkerOutput::SectionUpdateRequiredChanged(_)
                | ReaderWorkerOutput::VerseEntitiesUpdated(_)
                | ReaderWorkerOutput::TabsUpdated { .. } => {}
            },
            StudyViewInput::SelectTranslation(translation) => {
                self.selected_translation_id = Some(translation.id.clone());
                self.update_header_state();
                self.reader_worker
                    .emit(ReaderWorkerInput::SelectTranslation(translation));
            }
            StudyViewInput::SelectBook(book_id) => {
                self.selected_book_id = Some(book_id.clone());
                self.selected_chapter = Some(1);
                self.update_header_state();
                self.refresh_chapter_list(&widgets.chapter_list, &sender);
                self.reader_worker
                    .emit(ReaderWorkerInput::SelectBook(book_id));
            }
            StudyViewInput::SelectChapter(chapter) => {
                self.selected_chapter = Some(chapter);
                self.update_header_state();
                if let Some(book_id) = self.selected_book_id.clone() {
                    self.reader_worker
                        .emit(ReaderWorkerInput::OpenChapter { book_id, chapter });
                }
            }
            StudyViewInput::OpenStore => {
                let _ = sender.output(StudyViewOutput::OpenStoreRequested);
            }
            StudyViewInput::RefreshVersions => {
                self.reader_worker
                    .emit(ReaderWorkerInput::ReloadInstalledTranslations);
            }
            StudyViewInput::PreviousChapter => {
                self.reader_worker.emit(ReaderWorkerInput::PreviousChapter);
            }
            StudyViewInput::NextChapter => {
                self.reader_worker.emit(ReaderWorkerInput::NextChapter);
            }
            StudyViewInput::OpenChapterPicker => {
                widgets.chapter_menu.popup();
            }
            StudyViewInput::OpenThemePopup => {
                let _ = sender.output(StudyViewOutput::ToggleSidebar);
            }
            StudyViewInput::ChapterEvent(event) => match event {
                ChapterViewEvent::VerseAction {
                    book_id,
                    chapter,
                    verse,
                    action,
                } => match action {
                    VerseContextAction::Highlight(color) => {
                        let highlight = PersistentHighlight::new(
                            &book_id,
                            chapter as u32,
                            verse as u32,
                            color,
                            HighlightStyle::Solid,
                        );
                        if let Err(error) = self.highlights.upsert(highlight) {
                            let _ = sender.output(StudyViewOutput::ShowError(error));
                        } else {
                            let _ = sender
                                .output(StudyViewOutput::HighlightChanged(self.highlights.clone()));
                            self.refresh_all_tabs(&sender);
                        }
                    }
                    VerseContextAction::ClearHighlight => {
                        if let Err(error) =
                            self.highlights
                                .remove(&book_id, chapter as u32, verse as u32)
                        {
                            let _ = sender.output(StudyViewOutput::ShowError(error));
                        } else {
                            let _ = sender
                                .output(StudyViewOutput::HighlightChanged(self.highlights.clone()));
                            self.refresh_all_tabs(&sender);
                        }
                    }
                    VerseContextAction::Study(StudyToolKind::Commentary) => {
                        if let Some(page) = widgets.tab_view.selected_page()
                            && let Some(index) = self.tabs.iter().position(|slot| slot.page == page)
                        {
                            self.tabs[index].mode = StudyPanelMode::Commentary;
                            self.refresh_selected_slot(index, &sender);
                        }
                    }
                    VerseContextAction::Study(
                        StudyToolKind::Dictionary | StudyToolKind::Lexicon,
                    ) => {
                        self.selected_verse = Some(verse);
                        self.refresh_all_tabs(&sender);
                    }
                    VerseContextAction::Copy | VerseContextAction::Share => {
                        let _ = sender.output(StudyViewOutput::CopyVerseRequested {
                            book_id,
                            chapter,
                            verse,
                        });
                    }
                },
                ChapterViewEvent::EntitySelected(entity) => {
                    info!("Selected entity: {} ({})", entity.name, entity.kind)
                }
            },
            StudyViewInput::SearchEvent(event) => match event {
                SearchPanelEvent::QueryChanged(query) => {
                    self.search_query = query.clone();
                    let _ = sender.output(StudyViewOutput::SearchQueryChanged(query));
                }
                SearchPanelEvent::ClearQuery => {
                    self.search_query.clear();
                    self.search_results = None;
                    self.search_error = None;
                    let _ = sender.output(StudyViewOutput::ClearSearchRequested);
                    self.refresh_all_tabs(&sender);
                }
                SearchPanelEvent::SearchTypeChanged(search_type) => {
                    self.search_type = search_type;
                    let _ = sender.output(StudyViewOutput::SearchTypeChanged(search_type));
                }
                SearchPanelEvent::ResetAllSections => {
                    self.selected_sections = CanonSection::all_set();
                    let _ = sender.output(StudyViewOutput::ResetSearchSectionsRequested);
                    self.refresh_all_tabs(&sender);
                }
                SearchPanelEvent::ToggleSection(section) => {
                    if self.selected_sections.contains(&section) {
                        if self.selected_sections.len() > 1 {
                            self.selected_sections.remove(&section);
                        }
                    } else {
                        self.selected_sections.insert(section);
                    }
                    let _ = sender.output(StudyViewOutput::ToggleSearchSectionRequested(section));
                    self.refresh_all_tabs(&sender);
                }
                SearchPanelEvent::BookFilterChanged(book_id) => {
                    if let Some(page) = widgets.tab_view.selected_page()
                        && let Some(index) = self.tabs.iter().position(|slot| slot.page == page)
                    {
                        self.tabs[index].book_filter = book_id;
                        self.refresh_selected_slot(index, &sender);
                    }
                }
                SearchPanelEvent::HitSelected(hit) => {
                    if let Some(page) = widgets.tab_view.selected_page()
                        && let Some(index) = self.tabs.iter().position(|slot| slot.page == page)
                    {
                        self.tabs[index].mode = StudyPanelMode::Read;
                        self.refresh_selected_slot(index, &sender);
                    }
                    let _ = sender.output(StudyViewOutput::OpenChapterRequested {
                        book_id: hit.book_id,
                        chapter: hit.chapter,
                    });
                }
            },
            StudyViewInput::CommentarySelected(id, name) => {
                self.study_commentary_id = Some(id.clone());
                self.study_commentary_name = name.clone();
                let _ = sender.output(StudyViewOutput::SelectCommentaryRequested { id, name });
            }
        }
    }
}

impl StudyView {
    fn sync_selection_from_chapter(&mut self) {
        if let Some(chapter) = self.current_chapter.as_ref() {
            self.selected_translation_id = Some(chapter.translation_id.clone());
            self.selected_book_id = Some(chapter.book_id.clone());
            self.selected_chapter = Some(chapter.chapter_number);
        }
    }

    pub(crate) fn tab_kind_from_worker(kind: ReaderTabKind) -> StudyPanelMode {
        match kind {
            ReaderTabKind::Read => StudyPanelMode::Read,
            ReaderTabKind::Search => StudyPanelMode::Search,
        }
    }

    fn refresh_version_list(&self, version_list: &gtk::Box, sender: &ComponentSender<Self>) {
        while let Some(child) = version_list.first_child() {
            version_list.remove(&child);
        }
        if self.installed_translations.is_empty() {
            let label = gtk::Label::builder()
                .label(&gettext("No installed versions"))
                .build();
            label.add_css_class("dim-label");
            version_list.append(&label);
            let button = gtk::Button::with_label(&gettext("Go to Store"));
            let tx = sender.output_sender().clone();
            button.connect_clicked(move |_| {
                let _ = tx.send(StudyViewOutput::OpenStoreRequested);
            });
            version_list.append(&button);
            return;
        }
        for translation in &self.installed_translations {
            let label = if translation.short_name.is_empty() {
                translation.name.clone()
            } else {
                format!("{} — {}", translation.short_name, translation.name)
            };
            let button = gtk::Button::with_label(&label);
            button.add_css_class("flat");
            let tx = sender.input_sender().clone();
            let translation = translation.clone();
            button.connect_clicked(move |_| {
                let _ = tx.send(StudyViewInput::SelectTranslation(translation.clone()));
            });
            version_list.append(&button);
        }
    }

    fn refresh_book_list(&self, book_list: &gtk::FlowBox, sender: &ComponentSender<Self>) {
        while let Some(child) = book_list.first_child() {
            book_list.remove(&child);
        }
        for book in self
            .sections
            .iter()
            .flat_map(|section| section.books.iter())
        {
            let button = gtk::Button::with_label(&book.name);
            button.add_css_class("flat");
            let tx = sender.input_sender().clone();
            let book_id = book.id.clone();
            button.connect_clicked(move |_| {
                let _ = tx.send(StudyViewInput::SelectBook(book_id.clone()));
            });
            book_list.insert(&button, -1);
        }
    }

    fn refresh_chapter_list(&self, chapter_list: &gtk::FlowBox, sender: &ComponentSender<Self>) {
        while let Some(child) = chapter_list.first_child() {
            chapter_list.remove(&child);
        }
        let Some(book) = self
            .active_header_state
            .as_ref()
            .and_then(|state| state.book.as_ref())
        else {
            return;
        };
        for chapter in 1..=book.number_of_chapters {
            let button = gtk::Button::with_label(&chapter.to_string());
            button.add_css_class("flat");
            let tx = sender.input_sender().clone();
            button.connect_clicked(move |_| {
                let _ = tx.send(StudyViewInput::SelectChapter(chapter));
            });
            chapter_list.insert(&button, -1);
        }
    }

    fn update_header_state(&mut self) {
        let translation = self
            .selected_translation_id
            .as_ref()
            .and_then(|id| {
                self.installed_translations
                    .iter()
                    .find(|item| &item.id == id)
                    .cloned()
            })
            .or_else(|| {
                self.current_chapter.as_ref().and_then(|chapter| {
                    self.installed_translations
                        .iter()
                        .find(|item| item.id == chapter.translation_id)
                        .cloned()
                })
            });
        let book = self
            .selected_book_id
            .as_ref()
            .and_then(|id| {
                self.sections
                    .iter()
                    .flat_map(|section| section.books.iter())
                    .find(|item| &item.id == id)
                    .cloned()
            })
            .or_else(|| {
                self.current_chapter.as_ref().and_then(|chapter| {
                    self.sections
                        .iter()
                        .flat_map(|section| section.books.iter())
                        .find(|item| item.id == chapter.book_id)
                        .cloned()
                })
            });
        let chapter = self.selected_chapter.or_else(|| {
            self.current_chapter
                .as_ref()
                .map(|chapter| chapter.chapter_number)
        });
        self.active_header_state = Some(HeaderState {
            module: translation,
            book,
            chapter,
        });
    }

    fn update_header_state_with_translation(&mut self, translation: Option<GenericTranslation>) {
        let book = self
            .selected_book_id
            .as_ref()
            .and_then(|id| {
                self.sections
                    .iter()
                    .flat_map(|section| section.books.iter())
                    .find(|item| &item.id == id)
                    .cloned()
            })
            .or_else(|| {
                self.current_chapter.as_ref().and_then(|chapter| {
                    self.sections
                        .iter()
                        .flat_map(|section| section.books.iter())
                        .find(|item| item.id == chapter.book_id)
                        .cloned()
                })
            });
        let chapter = self.selected_chapter.or_else(|| {
            self.current_chapter
                .as_ref()
                .map(|chapter| chapter.chapter_number)
        });
        self.active_header_state = Some(HeaderState {
            module: translation.or_else(|| {
                self.current_chapter.as_ref().and_then(|chapter| {
                    self.installed_translations
                        .iter()
                        .find(|item| item.id == chapter.translation_id)
                        .cloned()
                })
            }),
            book,
            chapter,
        });
    }

    fn append_tab(
        &mut self,
        tab_view: &adw::TabView,
        title: &str,
        mode: StudyPanelMode,
        select: bool,
        sender: &ComponentSender<Self>,
    ) {
        let stack = gtk::Stack::builder().hexpand(true).vexpand(true).build();
        let read_overlay = gtk::Overlay::new();
        read_overlay.set_hexpand(true);
        read_overlay.set_vexpand(true);
        let read_scroll = gtk::ScrolledWindow::builder()
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vexpand(true)
            .hexpand(true)
            .build();
        read_overlay.set_child(Some(&read_scroll));
        let page_control = self.page_controls.widget();
        read_overlay.add_overlay(page_control);
        stack.add_named(&read_overlay, Some("read"));
        let page = tab_view.append(&stack);
        page.set_title(title);
        if select {
            tab_view.set_selected_page(&page);
        }
        let mut slot = TabSlot {
            page,
            stack,
            mode,
            read_overlay,
            read_scroll,
            book_filter: None,
        };
        self.refresh_slot(&mut slot, sender);
        self.tabs.push(slot);
    }

    fn refresh_all_tabs(&mut self, sender: &ComponentSender<Self>) {
        let mut tabs = std::mem::take(&mut self.tabs);
        for slot in &mut tabs {
            self.refresh_slot(slot, sender);
        }
        self.tabs = tabs;
    }

    fn refresh_selected_slot(&mut self, index: usize, sender: &ComponentSender<Self>) {
        let mut tabs = std::mem::take(&mut self.tabs);
        if let Some(slot) = tabs.get_mut(index) {
            self.refresh_slot(slot, sender);
        }
        self.tabs = tabs;
    }

    fn refresh_slot(&self, slot: &mut TabSlot, sender: &ComponentSender<Self>) {
        match slot.mode {
            StudyPanelMode::Read => {
                let scroll = &slot.read_scroll;
                if self.is_loading_content {
                    let spinner_box = gtk::Box::builder()
                        .orientation(gtk::Orientation::Vertical)
                        .halign(gtk::Align::Center)
                        .valign(gtk::Align::Center)
                        .vexpand(true)
                        .build();
                    let spinner = gtk::Spinner::new();
                    spinner.set_spinning(true);
                    spinner.set_width_request(32);
                    spinner.set_height_request(32);
                    spinner_box.append(&spinner);
                    scroll.set_child(Some(&spinner_box));
                } else if let Some(chapter) = &self.current_chapter {
                    let tx = sender.input_sender().clone();
                    let chapter_view = build_chapter_view(
                        chapter,
                        &self.text_config,
                        &self.highlights,
                        &self.chapter_entities,
                        self.selected_verse,
                        move |event| {
                            let _ = tx.send(StudyViewInput::ChapterEvent(event));
                        },
                    );
                    scroll.set_child(Some(&chapter_view));

                    self.page_controls.emit(crate::features::study::study_comonents::page_control_button::ThemeMenuInput::SetNavigationState {
                        has_prev: chapter.prev_chapter.is_some() || chapter.chapter_number > 1,
                        has_next: chapter.next_chapter.is_some() || self.sections.iter().flat_map(|section| section.books.iter()).any(|book| book.id == chapter.book_id && chapter.chapter_number < book.number_of_chapters),
                        chapter_label: format!("{} {}", chapter.book_name, chapter.chapter_number),
                    });
                    slot.page
                        .set_title(&format!("{} {}", chapter.book_name, chapter.chapter_number));
                } else if self.installed_translations.is_empty() {
                    let placeholder = gtk::Box::builder()
                        .orientation(gtk::Orientation::Vertical)
                        .spacing(10)
                        .halign(gtk::Align::Center)
                        .valign(gtk::Align::Center)
                        .hexpand(true)
                        .vexpand(true)
                        .build();
                    let heading = gtk::Label::builder()
                        .label(&gettext("No Bible Versions Installed"))
                        .build();
                    heading.add_css_class("title-2");
                    let description = gtk::Label::builder()
                        .label(&gettext(
                            "Visit the Store to install a Bible translation and start reading.",
                        ))
                        .wrap(true)
                        .justify(gtk::Justification::Center)
                        .build();
                    description.add_css_class("dim-label");
                    let store_button = gtk::Button::with_label(&gettext("Go to Store"));
                    store_button.add_css_class("suggested-action");
                    let tx = sender.output_sender().clone();
                    store_button.connect_clicked(move |_| {
                        let _ = tx.send(StudyViewOutput::OpenStoreRequested);
                    });
                    placeholder.append(&heading);
                    placeholder.append(&description);
                    placeholder.append(&store_button);
                    scroll.set_child(Some(&placeholder));
                } else {
                    let placeholder = gtk::Box::builder()
                        .orientation(gtk::Orientation::Vertical)
                        .spacing(10)
                        .halign(gtk::Align::Center)
                        .valign(gtk::Align::Center)
                        .hexpand(true)
                        .vexpand(true)
                        .build();
                    let heading = gtk::Label::builder()
                        .label(&gettext("Study Reader"))
                        .halign(gtk::Align::Center)
                        .build();
                    heading.add_css_class("title-2");
                    let subtitle = gtk::Label::builder()
                        .label(&gettext(
                            "Select a translation and chapter to begin reading.",
                        ))
                        .halign(gtk::Align::Center)
                        .build();
                    subtitle.add_css_class("dim-label");
                    placeholder.append(&heading);
                    placeholder.append(&subtitle);
                    scroll.set_child(Some(&placeholder));
                }
                slot.stack.set_visible_child_name("read");
            }
            StudyPanelMode::Commentary => {
                let tx = sender.input_sender().clone();
                let panel = build_commentary_panel(
                    &self.commentaries,
                    self.study_commentary_id.as_deref(),
                    &self.study_commentary_name,
                    self.study_commentary_data.as_ref(),
                    self.study_commentary_loading,
                    &self.text_config,
                    move |id, name| {
                        let _ = tx.send(StudyViewInput::CommentarySelected(id, name));
                    },
                );
                if let Some(existing) = slot.stack.child_by_name("commentary") {
                    slot.stack.remove(&existing);
                }
                slot.stack.add_named(&panel, Some("commentary"));
                slot.stack.set_visible_child_name("commentary");
            }
            StudyPanelMode::Search => {
                let tx = sender.input_sender().clone();
                let panel = build_search_panel(
                    &self.search_query,
                    self.search_type,
                    &self.selected_sections,
                    slot.book_filter.as_deref(),
                    &self.sections,
                    self.is_searching,
                    self.search_progress_fraction,
                    &self.search_progress_message,
                    self.search_results.as_ref(),
                    self.search_error.as_deref(),
                    move |event| {
                        let _ = tx.send(StudyViewInput::SearchEvent(event));
                    },
                );
                if let Some(existing) = slot.stack.child_by_name("search") {
                    slot.stack.remove(&existing);
                }
                slot.stack.add_named(&panel, Some("search"));
                slot.stack.set_visible_child_name("search");
            }
        }
    }
}
