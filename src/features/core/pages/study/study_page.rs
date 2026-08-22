use adw::prelude::*;
use relm4::prelude::*;
use std::sync::Arc;

use xbible_engine::engines::{
    module_engine::sword_module::{module::SwordModule, module_book::ModuleBook},
    xbible_engine::engine::XBibleEngine,
};

use crate::features::core::pages::study::bible_page_component::biblepage_root::{
    BiblePageRoot, BiblePageRootInput, BiblePageRootOutput, HeaderState,
};

struct TabEntry {
    controller: Controller<BiblePageRoot>,
    page: adw::TabPage,
    header_state: HeaderState,
}

pub struct StudyPage {
    engine: Arc<XBibleEngine>,
    tabs: Vec<TabEntry>,
    current_reference: String,
    active_header_state: Option<HeaderState>,
}

#[derive(Debug)]
pub enum StudyPageInput {
    UpdateTheme,
    NewTab,
    TabSelected(adw::TabPage),
    ReferenceChanged(String),
    SubmitReference(String),
    TabHeaderStateChanged(HeaderState),
    SetModule(SwordModule),
    SetBook(ModuleBook),
    SetChapter(i32),
    RefreshGrids,
}

#[derive(Debug)]
pub enum StudyPageOutput {
    ToggleSidebar,
}

#[relm4::component(pub)]
impl Component for StudyPage {
    type Init = (Arc<XBibleEngine>, bool);
    type Input = StudyPageInput;
    type Output = StudyPageOutput;
    type CommandOutput = ();

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        info!("reached study page init");
        let (engine, _is_sidebar_visible) = init;

        let modules = engine.get_bible_modules();
        let first_module = modules.first().cloned();
        let first_book = first_module.as_ref().and_then(|m| {
            let books = engine.get_books(m.name.as_str());
            books.first().cloned()
        });
        let first_chapter = first_book
            .as_ref()
            .and_then(|b| b.chapters.first().cloned())
            .map(|c| c.number);

        let initial_header_state = if first_module.is_some() {
            Some(HeaderState {
                module: first_module,
                book: first_book,
                chapter: first_chapter,
            })
        } else {
            None
        };

        let mut model = StudyPage {
            engine: engine.clone(),
            tabs: Vec::new(),
            current_reference: String::new(),
            active_header_state: initial_header_state.clone(),
        };

        let widgets = view_output!();

        if let Some(state) = initial_header_state {
            model.spawn_tab(&widgets.tab_view, &sender, state);
            model.refresh_all_grids(&widgets, sender.clone());
        }

        info!("StudyPage initialized with {} tabs", model.tabs.len());
        ComponentParts { model, widgets }
    }

    view! {
        adw::NavigationPage {
            set_title: "Study",
            #[wrap(Some)]
            set_child = &adw::TabOverview {
                #[wrap(Some)]
                set_child = &adw::ToolbarView {
                    #[wrap(Some)]
                    #[name = "tab_view"]
                    set_content = &adw::TabView {
                        connect_selected_page_notify[sender] => move |view| {
                            if let Some(page) = view.selected_page() {
                                sender.input(StudyPageInput::TabSelected(page));
                            }
                        },
                    },
                    add_top_bar = &adw::HeaderBar {
                        pack_start = &gtk::ToggleButton {
                            set_icon_name: "sidebar-show-symbolic",
                            connect_clicked[sender] => move |_| {
                                let _ = sender.output(StudyPageOutput::ToggleSidebar);
                            }
                        },

                        #[wrap(Some)]
                        #[name = "title_widget"]
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
                                    #[name = "version_label"]
                                    gtk::Label {
                                        #[watch]
                                        set_label: model.active_header_state
                                            .as_ref()
                                            .and_then(|s| s.module.as_ref())
                                            .map(|m| m.name.as_str())
                                            .unwrap_or("Select Version"),
                                    },
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

                                        #[name = "book_grid"]
                                        gtk::FlowBox {
                                            set_max_children_per_line: 4,
                                            set_min_children_per_line: 4,
                                            set_selection_mode: gtk::SelectionMode::None,
                                            set_column_spacing: 8,
                                            set_row_spacing: 8,
                                        },
                                    },
                                },
                                #[wrap(Some)]
                                set_child = &gtk::Box {
                                    set_spacing: 4,
                                    #[name = "book_label"]
                                    gtk::Label {
                                        #[watch]
                                        set_label: model.active_header_state
                                            .as_ref()
                                            .and_then(|s| s.book.as_ref())
                                            .map(|b| b.name.as_str())
                                            .unwrap_or("Select Book"),
                                    },
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
                                    gtk::Label {
                                        #[watch]
                                        set_label: &model.active_header_state
                                            .as_ref()
                                            .and_then(|s| s.chapter)
                                            .map(|c| format!("Chapter {}", c))
                                            .unwrap_or_default(),
                                    },
                                    gtk::Image { set_icon_name: Some("pan-down-symbolic") },
                                },
                            },
                        },

                        pack_end = &adw::TabButton {
                            set_view: Some(&tab_view),
                            set_action_name: Some("overview.open"),
                        },
                        pack_end = &gtk::Button {
                            set_icon_name: "tab-new-symbolic",
                            connect_clicked => StudyPageInput::NewTab,
                        }
                    },
                    add_top_bar = &adw::TabBar {
                        set_autohide: false,
                        set_view: Some(&tab_view),
                    },
                },
                set_view: Some(&tab_view),
            }
        }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            StudyPageInput::NewTab => {
                if let Some(active_state) = self.active_header_state.clone() {
                    self.spawn_tab(&widgets.tab_view, &sender, active_state);
                    self.refresh_all_grids(widgets, sender.clone());
                }
            }

            StudyPageInput::TabSelected(page) => {
                self.current_reference = page.title().to_string();
                if let Some(entry) = self.tabs.iter().find(|t| t.page == page) {
                    self.active_header_state = Some(entry.header_state.clone());
                    self.refresh_all_grids(widgets, sender.clone());
                }
            }

            StudyPageInput::TabHeaderStateChanged(state) => {
                if let Some(selected_page) = widgets.tab_view.selected_page() {
                    if let Some(entry) = self.tabs.iter_mut().find(|t| t.page == selected_page) {
                        entry.header_state = state.clone();
                        let title = Self::format_reference_title(&state);
                        entry.page.set_title(&title);
                        self.current_reference = title;
                    }
                }
                self.active_header_state = Some(state);
                self.refresh_all_grids(widgets, sender.clone());
            }

            StudyPageInput::SubmitReference(new_reference) => {
                let new_reference = new_reference.trim().to_string();
                if new_reference.is_empty() {
                    return;
                }
                if let Some(selected) = widgets.tab_view.selected_page() {
                    selected.set_title(&new_reference);
                    if let Some(entry) = self.tabs.iter().find(|t| t.page == selected) {
                        entry
                            .controller
                            .emit(BiblePageRootInput::GoToReference(new_reference.clone()));
                    }
                }
                self.current_reference = new_reference;
            }

            StudyPageInput::ReferenceChanged(reference) => {
                if let Some(selected) = widgets.tab_view.selected_page() {
                    let is_active_tab = self
                        .tabs
                        .iter()
                        .any(|t| t.page == selected && t.page.title() == self.current_reference);
                    if is_active_tab {
                        selected.set_title(&reference);
                        self.current_reference = reference;
                    }
                }
            }

            StudyPageInput::SetModule(sword_module) => {
                let books = self.engine.get_books(sword_module.name.as_str());
                let first_book = books.first().cloned();
                let first_chapter = first_book
                    .as_ref()
                    .and_then(|b| b.chapters.first().cloned())
                    .map(|c| c.number);

                let updated_state = HeaderState {
                    module: Some(sword_module),
                    book: first_book,
                    chapter: first_chapter,
                };

                let title = Self::format_reference_title(&updated_state);

                if let Some(selected_page) = widgets.tab_view.selected_page() {
                    selected_page.set_title(&title);
                    if let Some(entry) = self.tabs.iter_mut().find(|t| t.page == selected_page) {
                        entry.header_state = updated_state.clone();
                        entry
                            .controller
                            .emit(BiblePageRootInput::HeaderStateChanged(updated_state.clone()));
                    }
                }
                self.current_reference = title;
                self.active_header_state = Some(updated_state);
                self.refresh_all_grids(widgets, sender.clone());
            }

            StudyPageInput::SetBook(module_book) => {
                let first_chapter = module_book.chapters.first().map(|c| c.number);

                if let Some(selected_page) = widgets.tab_view.selected_page() {
                    if let Some(entry) = self.tabs.iter_mut().find(|t| t.page == selected_page) {
                        entry.header_state.book = Some(module_book);
                        entry.header_state.chapter = first_chapter;

                        let updated_state = entry.header_state.clone();
                        let title = Self::format_reference_title(&updated_state);

                        selected_page.set_title(&title);
                        self.current_reference = title;
                        self.active_header_state = Some(updated_state.clone());

                        entry
                            .controller
                            .emit(BiblePageRootInput::HeaderStateChanged(updated_state));
                    }
                }
                self.refresh_all_grids(widgets, sender.clone());
            }

            StudyPageInput::SetChapter(chapter) => {
                if let Some(selected_page) = widgets.tab_view.selected_page() {
                    if let Some(entry) = self.tabs.iter_mut().find(|t| t.page == selected_page) {
                        entry.header_state.chapter = Some(chapter);

                        let updated_state = entry.header_state.clone();
                        let title = Self::format_reference_title(&updated_state);

                        selected_page.set_title(&title);
                        self.current_reference = title;
                        self.active_header_state = Some(updated_state.clone());

                        entry
                            .controller
                            .emit(BiblePageRootInput::HeaderStateChanged(updated_state));
                    }
                }
                self.refresh_all_grids(widgets, sender.clone());
            }

            StudyPageInput::RefreshGrids => {
                self.refresh_all_grids(widgets, sender.clone());
            }

            StudyPageInput::UpdateTheme => {}
        }

        self.update_view(widgets, sender);
    }
}

impl StudyPage {
    fn format_reference_title(state: &HeaderState) -> String {
        match (&state.module, &state.book, state.chapter) {
            (Some(m), Some(b), Some(c)) => format!("{} {} {}", m.name, b.name, c),
            (Some(m), Some(b), None) => format!("{} {}", m.name, b.name),
            (Some(m), None, _) => m.name.clone(),
            _ => "New Tab".to_string(),
        }
    }

    fn refresh_all_grids(&self, widgets: &StudyPageWidgets, sender: ComponentSender<Self>) {
        let modules = self.engine.get_bible_modules();
        Self::populate_version_grid(widgets, &modules, sender.clone());

        if let Some(active_state) = &self.active_header_state {
            if let Some(module) = &active_state.module {
                let books = self.engine.get_books(module.name.as_str());
                Self::populate_book_grid(widgets, &books, sender.clone());
            }

            if let Some(book) = &active_state.book {
                Self::populate_chapter_grid(
                    widgets,
                    sender.clone(),
                    book.chapters.len() as i32,
                );
            }
        }
    }

    fn spawn_tab(
        &mut self,
        tab_view: &adw::TabView,
        sender: &ComponentSender<Self>,
        header_state: HeaderState,
    ) {
        let title = Self::format_reference_title(&header_state);

        let controller = BiblePageRoot::builder()
            .launch(self.engine.clone())
            .forward(sender.input_sender(), |message| match message {
                BiblePageRootOutput::UpdateTheme => StudyPageInput::UpdateTheme,
                BiblePageRootOutput::ReferenceChanged(reference) => {
                    StudyPageInput::ReferenceChanged(reference)
                }
                BiblePageRootOutput::HeaderStateChanged(state) => {
                    StudyPageInput::TabHeaderStateChanged(state)
                }
            });

        controller.emit(BiblePageRootInput::HeaderStateChanged(header_state.clone()));

        let page = tab_view.append(controller.widget());
        page.set_title(&title);
        tab_view.set_selected_page(&page);

        self.current_reference = title;
        self.active_header_state = Some(header_state.clone());
        self.tabs.push(TabEntry {
            controller,
            page,
            header_state,
        });
    }

    pub fn open_references(&self) -> Vec<String> {
        self.tabs
            .iter()
            .map(|t| t.page.title().to_string())
            .collect()
    }
}

impl StudyPage {
    pub(crate) fn populate_version_grid(
        widgets: &StudyPageWidgets,
        modules: &[SwordModule],
        sender: ComponentSender<Self>,
    ) {
        while let Some(child) = widgets.bible_grid.first_child() {
            widgets.bible_grid.remove(&child);
        }

        let mut grouped: std::collections::BTreeMap<String, Vec<SwordModule>> =
            std::collections::BTreeMap::new();
        for module in modules {
            grouped
                .entry(module.language.clone())
                .or_default()
                .push(module.clone());
        }

        let mut tasks: std::collections::VecDeque<(String, Vec<SwordModule>)> =
            grouped.into_iter().collect();

        let grid = widgets.bible_grid.clone();
        let pop = widgets.version_popover.clone();
        let s = sender.clone();

        glib::idle_add_local(move || {
            if let Some((lang, lang_modules)) = tasks.pop_front() {
                let header_label = gtk::Label::builder()
                    .halign(gtk::Align::Start)
                    .margin_top(20)
                    .margin_bottom(12)
                    .margin_start(16)
                    .build();

                header_label.set_markup(&format!(
                    "<span size='small' weight='heavy' alpha='60%' letter_spacing='1200'>{}</span>",
                    lang.to_uppercase()
                ));
                grid.append(&header_label);

                let wrap_box = adw::WrapBox::builder()
                    .orientation(gtk::Orientation::Horizontal)
                    .line_spacing(5)
                    .child_spacing(5)
                    .margin_start(10)
                    .margin_end(10)
                    .margin_bottom(20)
                    .build();

                for module in lang_modules {
                    let tile = Self::create_bible_tile(&module.name, &module.language);
                    let s_inner = s.clone();
                    let m_inner = module.clone();
                    let p_inner = pop.clone();

                    tile.connect_clicked(move |_| {
                        s_inner.input(StudyPageInput::SetModule(m_inner.clone()));
                        p_inner.popdown();
                    });
                    wrap_box.append(&tile);
                }
                grid.append(&wrap_box);

                let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
                sep.set_opacity(0.1);
                sep.set_margin_start(16);
                sep.set_margin_end(16);
                grid.append(&sep);

                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });
    }

    pub(crate) fn populate_book_grid(
        widgets: &StudyPageWidgets,
        books: &[ModuleBook],
        sender: ComponentSender<Self>,
    ) {
        while let Some(child) = widgets.book_grid.first_child() {
            widgets.book_grid.remove(&child);
        }

        let mut books_queue: std::collections::VecDeque<ModuleBook> = books.to_vec().into();

        let book_grid = widgets.book_grid.clone();
        let pop = widgets.book_popover.clone();
        let s = sender.clone();

        glib::idle_add_local(move || {
            for _ in 0..8 {
                if let Some(book) = books_queue.pop_front() {
                    let btn = Self::create_book_tile(&book.name);
                    let s_inner = s.clone();
                    let p_inner = pop.clone();
                    let book_for_click = book.clone();

                    btn.connect_clicked(move |_| {
                        s_inner.input(StudyPageInput::SetBook(book_for_click.clone()));
                        p_inner.popdown();
                    });

                    book_grid.append(&btn);
                    book_grid.set_visible(true);
                } else {
                    return glib::ControlFlow::Break;
                }
            }
            glib::ControlFlow::Continue
        });
    }

    pub(crate) fn populate_chapter_grid(
        widgets: &StudyPageWidgets,
        sender: ComponentSender<Self>,
        count: i32,
    ) {
        while let Some(child) = widgets.chapter_grid.first_child() {
            widgets.chapter_grid.remove(&child);
        }

        let mut current_idx = 1;
        let grid = widgets.chapter_grid.clone();
        let pop = widgets.chapter_popover.clone();
        let s = sender.clone();

        glib::idle_add_local(move || {
            for _ in 0..12 {
                if current_idx <= count {
                    let btn = gtk::Button::builder()
                        .label(&current_idx.to_string())
                        .css_classes(vec!["card", "chapter-tile"])
                        .build();

                    let s_inner = s.clone();
                    let p_inner = pop.clone();
                    let val = current_idx;

                    btn.connect_clicked(move |_| {
                        s_inner.input(StudyPageInput::SetChapter(val));
                        p_inner.popdown();
                    });

                    grid.append(&btn);
                    current_idx += 1;
                } else {
                    return glib::ControlFlow::Break;
                }
            }
            glib::ControlFlow::Continue
        });
    }

    pub(crate) fn create_bible_tile(name: &str, _lang: &str) -> gtk::Button {
        let button = gtk::Button::builder()
            .width_request(120)
            .height_request(48)
            .css_classes(vec!["card", "flat"])
            .build();

        let label = gtk::Label::builder().build();
        label.set_markup(&format!(
            "<span weight='bold' size='medium' font_features='tnum'>{}</span>",
            name
        ));

        button.set_child(Some(&label));
        button.set_margin_all(2);
        button
    }

    pub(crate) fn create_book_tile(name: &str) -> gtk::Button {
        gtk::Button::builder()
            .label(name)
            .css_classes(vec!["card", "book-tile"])
            .width_request(85)
            .height_request(40)
            .build()
    }
}