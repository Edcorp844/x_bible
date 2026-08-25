use std::sync::Arc;
use adw::prelude::*;
use gtk::prelude::*;
use relm4::prelude::*;

use xbible_engine::engines::{
    module_engine::{
        module_engine_extensions::module_engine_search_ext::SearchType,
        sword_module::module::SwordModule,
    },
    xbible_engine::engine::XBibleEngine,
};

#[derive(Debug, Clone)]
pub struct SearchHit {
    pub reference: String,
    pub text: String,
    pub book: String,
    pub chapter: i32,
    pub verse: i32,
    pub score: i64,
}

pub struct SearchPage {
    pub(crate) engine: Arc<XBibleEngine>,
    pub(crate) module: Option<SwordModule>,
    pub(crate) query: String,
    pub(crate) search_type: SearchType,
    pub(crate) results: Vec<SearchHit>,
    pub(crate) is_searching: bool,
    pub(crate) total_hits: usize,
}

#[derive(Debug, Clone)]
pub enum SearchPageInput {
    SetModule(SwordModule),
    UpdateQuery(String),
    SetSearchType(SearchType),
    ExecuteSearch,
    SearchCompleted(Vec<SearchHit>),
    OpenVerse(SearchHit),
    OpenVerseInNewTab(SearchHit),
}

#[derive(Debug, Clone)]
pub enum SearchPageOutput {
    UpdateTheme,
    NavigateToVerse {
        module: SwordModule,
        book: String,
        chapter: i32,
        verse: i32,
    },
    OpenVerseInNewTab {
        module: SwordModule,
        book: String,
        chapter: i32,
        verse: i32,
    },
}

#[relm4::component(pub)]
impl Component for SearchPage {
    type Init = (Arc<XBibleEngine>, Option<SwordModule>);
    type Input = SearchPageInput;
    type Output = SearchPageOutput;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,
            set_vexpand: true,

            // Top Search Header Area
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                add_css_class: "background",
                set_margin_start: 16,
                set_margin_end: 16,
                set_margin_top: 16,
                set_margin_bottom: 8,

                adw::Clamp {
                    set_maximum_size: 800,
                    set_tightening_threshold: 600,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 12,

                        // Search Entry
                        adw::PreferencesGroup {
                            adw::EntryRow {
                                set_title: "Search Scriptures",
                                set_show_apply_button: true,
                                connect_changed[sender] => move |entry| {
                                    sender.input(SearchPageInput::UpdateQuery(entry.text().to_string()));
                                },
                                connect_entry_activated[sender] => move |_| {
                                    sender.input(SearchPageInput::ExecuteSearch);
                                },
                                connect_apply[sender] => move |_| {
                                    sender.input(SearchPageInput::ExecuteSearch);
                                },
                            },
                        },

                        // Mode Selector & Status Bar
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 12,
                            set_margin_start: 4,
                            set_margin_end: 4,

                            // Search Mode Filter Dropdown
                            gtk::DropDown {
                                set_model: Some(&gtk::StringList::new(&[
                                    "Multi-Word",
                                    "Phrase",
                                    "Regex",
                                ])),
                                set_selected: 0,
                                set_valign: gtk::Align::Center,
                                connect_selected_notify[sender] => move |dropdown| {
                                    let mode = match dropdown.selected() {
                                        1 => SearchType::Phrase,
                                        2 => SearchType::RegularExpression,
                                        _ => SearchType::MultiWord,
                                    };
                                    sender.input(SearchPageInput::SetSearchType(mode));
                                },
                            },

                            // Module Tag Indicator
                            gtk::Label {
                                #[watch]
                                set_markup: &format!(
                                    "<span size='small' weight='bold' alpha='60%'>MODULE: {}</span>",
                                    model.module.as_ref().map(|m| m.name.as_str()).unwrap_or("None")
                                ),
                                set_valign: gtk::Align::Center,
                            },

                            gtk::Box { set_hexpand: true },

                            // Processing Spinner
                            gtk::Spinner {
                                #[watch]
                                set_spinning: model.is_searching,
                                #[watch]
                                set_visible: model.is_searching,
                            },

                            // Match Counter
                            gtk::Label {
                                #[watch]
                                set_markup: &if model.is_searching {
                                    "<span size='small' alpha='60%'>Searching engine…</span>".to_string()
                                } else if !model.query.is_empty() && model.results.is_empty() {
                                    "<span size='small' alpha='60%'>No results found</span>".to_string()
                                } else if !model.results.is_empty() {
                                    format!("<span size='small' weight='bold' alpha='70%'>{} results</span>", model.total_hits)
                                } else {
                                    String::new()
                                },
                                set_valign: gtk::Align::Center,
                            },
                        },
                    },
                },
            },

            gtk::Separator {
                set_orientation: gtk::Orientation::Horizontal,
                set_opacity: 0.15,
            },

            // Results Container
            gtk::ScrolledWindow {
                set_hscrollbar_policy: gtk::PolicyType::Never,
                set_vexpand: true,
                set_hexpand: true,

                adw::Clamp {
                    set_maximum_size: 800,
                    set_tightening_threshold: 600,
                    set_margin_top: 16,
                    set_margin_bottom: 24,
                    set_margin_start: 16,
                    set_margin_end: 16,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 12,

                        // Empty State Display
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_valign: gtk::Align::Center,
                            set_halign: gtk::Align::Center,
                            set_spacing: 12,
                            set_margin_top: 60,
                            #[watch]
                            set_visible: model.results.is_empty() && !model.is_searching,

                            gtk::Image {
                                set_icon_name: Some("edit-find-symbolic"),
                                set_pixel_size: 64,
                                add_css_class: "dim-label",
                            },
                            gtk::Label {
                                set_markup: "<span size='large' weight='bold'>Search the Bible</span>",
                                add_css_class: "dim-label",
                            },
                            gtk::Label {
                                set_label: "Enter keywords, phrases, or topics to query the active translation.",
                                add_css_class: "dim-label",
                                set_wrap: true,
                                set_justify: gtk::Justification::Center,
                            },
                        },

                        // Result Item Box
                        #[name = "results_box"]
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 16,
                        },
                    },
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (engine, module) = init;

        let model = Self {
            engine,
            module,
            query: String::new(),
            search_type: SearchType::MultiWord,
            results: Vec::new(),
            is_searching: false,
            total_hits: 0,
        };

        let widgets = view_output!();

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
            SearchPageInput::SetModule(module) => {
                self.module = Some(module);
                if !self.query.trim().is_empty() {
                    sender.input(SearchPageInput::ExecuteSearch);
                }
            }

            SearchPageInput::UpdateQuery(q) => {
                self.query = q;
            }

            SearchPageInput::SetSearchType(st) => {
                self.search_type = st;
                if !self.query.trim().is_empty() {
                    sender.input(SearchPageInput::ExecuteSearch);
                }
            }

            SearchPageInput::ExecuteSearch => {
                let query = self.query.trim().to_string();
                let module = match &self.module {
                    Some(m) if !query.is_empty() => m.clone(),
                    _ => return,
                };

                self.is_searching = true;
                let engine = self.engine.clone();
                let search_type = self.search_type;
                let s = sender.clone();

                glib::spawn_future_local(async move {
                    let (tx, rx) = std::sync::mpsc::channel::<Vec<SearchHit>>();

                    std::thread::spawn(move || {
                        let search_results = engine.search(module.name.clone(), query, search_type);

                        let hits: Vec<SearchHit> = search_results
                            .hits
                            .into_iter()
                            .map(|h| {
                                let (book, chapter, verse) = Self::parse_sword_key(&h.key);
                                let sections = engine.get_single_entry(&module, &h.key);

                                let text = sections
                                    .into_iter()
                                    .flat_map(|s| s.verses)
                                    .flat_map(|v| v.words)
                                    .fold(String::new(), |mut acc, w| {
                                        if !acc.is_empty() && !w.is_punctuation {
                                            acc.push(' ');
                                        }
                                        acc.push_str(&w.text);
                                        acc
                                    });

                                SearchHit {
                                    reference: h.key,
                                    text,
                                    book,
                                    chapter,
                                    verse,
                                    score: h.score,
                                }
                            })
                            .collect();

                        let _ = tx.send(hits);
                    });

                    if let Ok(hits) = rx.recv() {
                        s.input(SearchPageInput::SearchCompleted(hits));
                    }
                });
            }

            SearchPageInput::SearchCompleted(hits) => {
                self.is_searching = false;
                self.total_hits = hits.len();
                self.results = hits;

                while let Some(child) = widgets.results_box.first_child() {
                    widgets.results_box.remove(&child);
                }

                let total = self.results.len();
                for (idx, hit) in self.results.iter().enumerate() {
                    let item_box = Self::create_result_item(hit, &self.query, sender.clone());
                    widgets.results_box.append(&item_box);

                    // Optional subtle separator line between list items
                    if idx < total - 1 {
                        let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
                        separator.set_opacity(0.1);
                        widgets.results_box.append(&separator);
                    }
                }
            }

            SearchPageInput::OpenVerse(hit) => {
                if let Some(module) = &self.module {
                    let _ = sender.output(SearchPageOutput::NavigateToVerse {
                        module: module.clone(),
                        book: hit.book,
                        chapter: hit.chapter,
                        verse: hit.verse,
                    });
                }
            }

            SearchPageInput::OpenVerseInNewTab(hit) => {
                if let Some(module) = &self.module {
                    let _ = sender.output(SearchPageOutput::OpenVerseInNewTab {
                        module: module.clone(),
                        book: hit.book,
                        chapter: hit.chapter,
                        verse: hit.verse,
                    });
                }
            }
        }
    }
}

impl SearchPage {
    fn parse_sword_key(key: &str) -> (String, i32, i32) {
        let parts: Vec<&str> = key.rsplitn(2, ' ').collect();
        if parts.len() == 2 {
            let book = parts[1].to_string();
            let chapter_verse: Vec<&str> = parts[0].split(':').collect();
            if chapter_verse.len() == 2 {
                let chapter = chapter_verse[0].parse::<i32>().unwrap_or(1);
                let verse = chapter_verse[1].parse::<i32>().unwrap_or(1);
                return (book, chapter, verse);
            }
        }
        (key.to_string(), 1, 1)
    }

    fn highlight_matches(text: &str, query: &str) -> String {
        if query.trim().is_empty() {
            return glib::markup_escape_text(text).to_string();
        }

        let escaped_text = glib::markup_escape_text(text).to_string();
        let terms: Vec<&str> = query.split_whitespace().collect();
        let mut result = escaped_text;

        for term in terms {
            let escaped_term = glib::markup_escape_text(term).to_string();
            result = result.replace(
                &escaped_term,
                &format!("<span background='#f6d32d' foreground='#000000'><b>{}</b></span>", escaped_term),
            );
        }

        result
    }

    fn create_result_item(
        hit: &SearchHit,
        query: &str,
        sender: ComponentSender<Self>,
    ) -> gtk::Box {
        let item_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(8)
            .margin_top(4)
            .margin_bottom(4)
            .build();

        // 1. Reference Line
        let ref_label = gtk::Label::builder()
            .halign(gtk::Align::Start)
            .build();
        ref_label.set_markup(&format!(
            "<span weight='bold' size='large'>{}</span>",
            glib::markup_escape_text(&hit.reference)
        ));

        // 2. Body Text Area
        let body_label = gtk::Label::builder()
            .halign(gtk::Align::Start)
            .wrap(true)
            .max_width_chars(75)
            .build();
        body_label.set_markup(&Self::highlight_matches(&hit.text, query));

        // 3. Action Buttons Row
        let action_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .margin_top(4)
            .build();

        // Copy Text Button
        let copy_button = gtk::Button::builder()
            .icon_name("edit-copy-symbolic")
            .css_classes(vec!["flat", "circular"])
            .tooltip_text("Copy verse text")
            .build();

        let text_to_copy = format!("{} - {}", hit.reference, hit.text);
        copy_button.connect_clicked(move |btn| {
            btn.clipboard().set_text(&text_to_copy);
        });

        let spacer = gtk::Box::builder().hexpand(true).build();

        // Open Verse Button (Navigates in active tab)
        let open_button = gtk::Button::builder()
            .label("Open")
            .css_classes(vec!["flat"])
            .build();

        let hit_open = hit.clone();
        let sender_open = sender.clone();
        open_button.connect_clicked(move |_| {
            sender_open.input(SearchPageInput::OpenVerse(hit_open.clone()));
        });

        // Open in New Tab Button
        let new_tab_button = gtk::Button::builder()
            .label("Open in New Tab")
            .icon_name("tab-new-symbolic")
            .css_classes(vec!["flat"])
            .build();

        let hit_tab = hit.clone();
        let sender_tab = sender.clone();
        new_tab_button.connect_clicked(move |_| {
            sender_tab.input(SearchPageInput::OpenVerseInNewTab(hit_tab.clone()));
        });

        action_box.append(&copy_button);
        action_box.append(&spacer);
        action_box.append(&open_button);
        action_box.append(&new_tab_button);

        item_box.append(&ref_label);
        item_box.append(&body_label);
        item_box.append(&action_box);

        item_box
    }
}