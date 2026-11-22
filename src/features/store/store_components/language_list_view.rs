use std::collections::BTreeMap;

use adw::prelude::*;
use gettextrs::gettext;
use relm4::prelude::*;
use xbible_engine::features::store::LanguageFacet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageSortOrder {
    Native,
    English,
    Items,
}

impl LanguageSortOrder {
    fn label(&self) -> String {
        match self {
            Self::Native => gettext("Native Name"),
            Self::English => gettext("English Name"),
            Self::Items => gettext("Most Items"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LanguageListInit {
    pub languages: Vec<LanguageFacet>,
}

#[derive(Debug)]
pub struct LanguageListView {
    languages: Vec<LanguageFacet>,
    search_query: String,
    sort_order: LanguageSortOrder,
}

#[derive(Debug)]
pub enum LanguageListInput {
    SetLanguages(Vec<LanguageFacet>),
    SearchChanged(String),
    SetSortOrder(LanguageSortOrder),
    SelectLanguage(LanguageFacet),
}

#[derive(Debug, Clone)]
pub enum LanguageListOutput {
    Selected(LanguageFacet),
}

#[relm4::component(pub)]
impl Component for LanguageListView {
    type Init = LanguageListInit;
    type Input = LanguageListInput;
    type Output = LanguageListOutput;
    type CommandOutput = ();

    view! {
        adw::NavigationPage {
            set_title: &gettext("Languages"),

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    #[name="sort_btn"]
                    pack_end = &gtk::MenuButton {
                        set_icon_name: "view-sort-ascending-symbolic",
                        set_tooltip_text: Some(&gettext("Sort By")),
                    },

                    #[name = "search_entry"]
                    pack_end = &gtk::SearchEntry {
                        set_placeholder_text: Some(&gettext("Search languages")),
                        set_hexpand: true,
                        connect_search_changed[sender] => move |entry| {
                            sender.input(LanguageListInput::SearchChanged(entry.text().to_string()));
                        },
                    },
                },


                #[wrap(Some)]
                    set_content = &adw::Clamp {
                    set_maximum_size: 1500,
                    set_tightening_threshold: 1000,

                    #[wrap(Some)]
                    set_child = &gtk::ScrolledWindow {
                        set_hscrollbar_policy: gtk::PolicyType::Never,
                        set_vexpand: true,

                        #[name = "groups_box"]
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 18,
                            set_margin_start: 12,
                            set_margin_end: 12,
                            set_margin_top: 8,
                            set_margin_bottom: 24,
                        },
                    }
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            languages: init.languages,
            search_query: String::new(),
            sort_order: LanguageSortOrder::Native,
        };

        let widgets = view_output!();
        attach_sort_popover(&widgets.sort_btn, model.sort_order, &sender);
        model.rebuild_groups(&widgets.groups_box, &sender);

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
            LanguageListInput::SetLanguages(languages) => {
                self.languages = languages;
                self.rebuild_groups(&widgets.groups_box, &sender);
            }
            LanguageListInput::SearchChanged(query) => {
                self.search_query = query;
                self.rebuild_groups(&widgets.groups_box, &sender);
            }
            LanguageListInput::SetSortOrder(order) => {
                self.sort_order = order;
                attach_sort_popover(&widgets.sort_btn, self.sort_order, &sender);
                self.rebuild_groups(&widgets.groups_box, &sender);
            }
            LanguageListInput::SelectLanguage(lang) => {
                let _ = sender.output(LanguageListOutput::Selected(lang));
            }
        }
    }
}

impl LanguageListView {
    fn grouped_languages(&self) -> BTreeMap<String, Vec<LanguageFacet>> {
        let q = self.search_query.trim().to_lowercase();

        let mut filtered: Vec<LanguageFacet> = self
            .languages
            .iter()
            .filter(|lang| {
                if q.is_empty() {
                    return true;
                }
                lang.name.to_lowercase().contains(&q)
                    || lang.english_name.to_lowercase().contains(&q)
                    || lang.code.to_lowercase().contains(&q)
            })
            .cloned()
            .collect();

        match self.sort_order {
            LanguageSortOrder::Native => {
                filtered.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            }
            LanguageSortOrder::English => {
                filtered.sort_by(|a, b| {
                    display_english_name(a)
                        .to_lowercase()
                        .cmp(&display_english_name(b).to_lowercase())
                });
            }
            LanguageSortOrder::Items => {
                filtered.sort_by(|a, b| {
                    let count_a = a.translation_count + a.commentary_count;
                    let count_b = b.translation_count + b.commentary_count;
                    count_b.cmp(&count_a)
                });
            }
        }

        let mut groups: BTreeMap<String, Vec<LanguageFacet>> = BTreeMap::new();
        for lang in filtered {
            let source = match self.sort_order {
                LanguageSortOrder::English => display_english_name(&lang),
                LanguageSortOrder::Native | LanguageSortOrder::Items => lang.name.clone(),
            };

            let letter = source
                .chars()
                .next()
                .filter(|c| c.is_alphabetic())
                .map(|c| c.to_uppercase().to_string())
                .unwrap_or_else(|| "#".to_string());

            groups.entry(letter).or_default().push(lang);
        }

        groups
    }

    fn rebuild_groups(&self, container: &gtk::Box, sender: &ComponentSender<Self>) {
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        let groups = self.grouped_languages();
        if groups.is_empty() {
            let empty = adw::StatusPage::builder()
                .icon_name("system-search-symbolic")
                .title(&gettext("No Languages Found"))
                .description(&gettext("Try a different search filter."))
                .build();
            container.append(&empty);
            return;
        }

        for (letter, items) in groups {
            let group = adw::PreferencesGroup::builder().title(&letter).build();

            for lang in items {
                let native_trimmed = lang.name.trim();
                let title = if native_trimmed.is_empty() {
                    &lang.code
                } else {
                    native_trimmed
                };

                let row = adw::ActionRow::builder()
                    .title(title)
                    .activatable(true)
                    .build();

                if let Some(subtitle) = secondary_english_name(&lang) {
                    row.set_subtitle(&subtitle);
                }

                let total = lang.translation_count + lang.commentary_count;
                let suffix_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
                suffix_box.set_valign(gtk::Align::Center);

                let count_lbl = gtk::Label::new(Some(&total.to_string()));
                count_lbl.add_css_class("dim-label");

                let chevron = gtk::Image::from_icon_name("go-next-symbolic");
                chevron.add_css_class("dim-label");

                suffix_box.append(&count_lbl);
                suffix_box.append(&chevron);
                row.add_suffix(&suffix_box);

                let lang_clone = lang.clone();
                let sender = sender.clone();
                row.connect_activated(move |_| {
                    sender.input(LanguageListInput::SelectLanguage(lang_clone.clone()));
                });

                group.add(&row);
            }

            container.append(&group);
        }
    }
}

fn display_english_name(lang: &LanguageFacet) -> String {
    let trimmed = lang.english_name.trim();
    if trimmed.is_empty() {
        lang.name.clone()
    } else {
        trimmed.to_string()
    }
}

fn secondary_english_name(lang: &LanguageFacet) -> Option<String> {
    let native = lang.name.trim();
    let english = lang.english_name.trim();
    if english.is_empty() {
        return None;
    }
    if native.is_empty() || !english.eq_ignore_ascii_case(native) {
        Some(english.to_string())
    } else {
        None
    }
}

fn attach_sort_popover(
    button: &gtk::MenuButton,
    current: LanguageSortOrder,
    sender: &ComponentSender<LanguageListView>,
) {
    let popover = gtk::Popover::new();
    let box_layout = gtk::Box::new(gtk::Orientation::Vertical, 4);
    box_layout.set_margin_top(8);
    box_layout.set_margin_bottom(8);
    box_layout.set_margin_start(8);
    box_layout.set_margin_end(8);

    let header = gtk::Label::builder()
        .label(&gettext("Sort By"))
        .halign(gtk::Align::Start)
        .margin_start(6)
        .margin_bottom(4)
        .build();
    header.add_css_class("heading");
    box_layout.append(&header);

    for order in [
        LanguageSortOrder::Native,
        LanguageSortOrder::English,
        LanguageSortOrder::Items,
    ] {
        let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let lbl = gtk::Label::builder()
            .label(&order.label())
            .halign(gtk::Align::Start)
            .hexpand(true)
            .build();
        row_box.append(&lbl);

        if current == order {
            let check = gtk::Image::from_icon_name("object-select-symbolic");
            row_box.append(&check);
        }

        let btn = gtk::Button::new();
        btn.set_child(Some(&row_box));
        btn.add_css_class("flat");

        let pop_weak = popover.downgrade();
        let sender = sender.clone();
        btn.connect_clicked(move |_| {
            sender.input(LanguageListInput::SetSortOrder(order));
            if let Some(pop) = pop_weak.upgrade() {
                pop.popdown();
            }
        });

        box_layout.append(&btn);
    }

    popover.set_child(Some(&box_layout));
    button.set_popover(Some(&popover));
}
