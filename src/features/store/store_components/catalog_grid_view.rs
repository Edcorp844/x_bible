use std::collections::{BTreeMap, HashMap};

use adw::prelude::*;
use gettextrs::gettext;
use relm4::prelude::*;
use xbible_engine::features::store::{StoreItem, StoreShelf};

use crate::{
    features::store::{
        store_components::translation_card::{
            TranslationCard, TranslationCardInit, TranslationCardInput, TranslationCardOutput,
            TranslationCardStatus,
        },
        store_item_ui_extension::StoreItemUIExt,
    },
    workers::store_worker::InstallState,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogSortOrder {
    Name,
    Language,
}

impl CatalogSortOrder {
    fn label(&self) -> String {
        match self {
            Self::Name => gettext("Name"),
            Self::Language => gettext("Language"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CatalogGridInit {
    pub title: String,
    pub subtitle: Option<String>,
    pub initial_items: Vec<StoreItem>,
    pub shelves: Vec<StoreShelf>,
    pub active_install: Option<InstallState>,
}

#[derive(Debug)]
pub struct CatalogGridScreen {
    title: String,
    subtitle: Option<String>,
    initial_items: Vec<StoreItem>,
    shelves: Vec<StoreShelf>,
    active_install: Option<InstallState>,
    search_query: String,
    sort_order: CatalogSortOrder,
    filter_language: Option<String>,
    cards: HashMap<String, Controller<TranslationCard>>,
}

#[derive(Debug)]
pub enum CatalogGridInput {
    SyncState {
        shelves: Vec<StoreShelf>,
        active_install: Option<InstallState>,
    },
    SearchChanged(String),
    SetSortOrder(CatalogSortOrder),
    SetFilterLanguage(Option<String>),
    CardEvent(TranslationCardOutput),
}

#[derive(Debug, Clone)]
pub enum CatalogGridOutput {
    InstallItem(StoreItem),
    OpenItem(StoreItem),
    SelectItem(StoreItem),
}

#[relm4::component(pub)]
impl Component for CatalogGridScreen {
    type Init = CatalogGridInit;
    type Input = CatalogGridInput;
    type Output = CatalogGridOutput;
    type CommandOutput = ();

    view! {
        adw::NavigationPage {
            set_title: &model.title,

            #[wrap(Some)]
            set_child = &adw::ToolbarView {

                add_top_bar = &adw::HeaderBar {
                    #[wrap(Some)]
                    #[name="window_title"]
                    set_title_widget = &adw::WindowTitle {
                        set_title: &model.title,
                        set_subtitle: &model.subtitle_text(),
                    },

                    #[name = "filter_btn"]
                    pack_end = &gtk::MenuButton {
                        set_icon_name: "funnel-symbolic",
                        set_tooltip_text: Some(&gettext("Filter by Language")),
                    },

                    #[name = "sort_btn"]
                    pack_end = &gtk::MenuButton {
                        set_icon_name: "view-sort-ascending-symbolic",
                        set_tooltip_text: Some(&gettext("Sort By")),
                    },
                },

                add_top_bar = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_start: 12,
                    set_margin_end: 12,
                    set_margin_top: 8,
                    set_margin_bottom: 8,

                    #[name = "search_entry"]
                    gtk::SearchEntry {
                        set_placeholder_text: Some(&gettext("Search {}").replace("{}", &model.title)),
                        set_hexpand: true,
                        connect_search_changed[sender] => move |entry| {
                            sender.input(CatalogGridInput::SearchChanged(entry.text().to_string()));
                        },
                    },
                },

                #[wrap(Some)]
                set_content = &gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_vexpand: true,

                    #[name = "flow_box"]
                    gtk::FlowBox {
                        set_selection_mode: gtk::SelectionMode::None,
                        set_homogeneous: true,
                        set_min_children_per_line: 1,
                        set_max_children_per_line: 8,
                        set_column_spacing: 16,
                        set_row_spacing: 20,
                        set_margin_all: 16,
                        set_valign: gtk::Align::Start,
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
        let mut model = Self {
            title: init.title,
            subtitle: init.subtitle,
            initial_items: init.initial_items,
            shelves: init.shelves,
            active_install: init.active_install,
            search_query: String::new(),
            sort_order: CatalogSortOrder::Name,
            filter_language: None,
            cards: HashMap::new(),
        };

        let widgets = view_output!();
        attach_sort_popover(&widgets.sort_btn, model.sort_order, &sender);
        attach_filter_popover(
            &widgets.filter_btn,
            &model.language_options(),
            model.filter_language.as_deref(),
            &sender,
        );
        model.rebuild_grid(&widgets.flow_box, &sender);

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            CatalogGridInput::SyncState {
                shelves,
                active_install,
            } => {
                self.shelves = shelves;
                self.active_install = active_install;
                widgets.window_title.set_subtitle(&self.subtitle_text());
                self.sync_or_rebuild(&widgets.flow_box, &sender);
            }
            CatalogGridInput::SearchChanged(query) => {
                self.search_query = query;
                self.rebuild_grid(&widgets.flow_box, &sender);
            }
            CatalogGridInput::SetSortOrder(order) => {
                self.sort_order = order;
                attach_sort_popover(&widgets.sort_btn, self.sort_order, &sender);
                self.rebuild_grid(&widgets.flow_box, &sender);
            }
            CatalogGridInput::SetFilterLanguage(lang) => {
                self.filter_language = lang;
                attach_filter_popover(
                    &widgets.filter_btn,
                    &self.language_options(),
                    self.filter_language.as_deref(),
                    &sender,
                );
                self.rebuild_grid(&widgets.flow_box, &sender);
            }
            CatalogGridInput::CardEvent(event) => match event {
                TranslationCardOutput::Install(item) => {
                    let _ = sender.output(CatalogGridOutput::InstallItem(item));
                }
                TranslationCardOutput::Open(item) => {
                    let _ = sender.output(CatalogGridOutput::OpenItem(item));
                }
                TranslationCardOutput::Selected(item) => {
                    let _ = sender.output(CatalogGridOutput::SelectItem(item));
                }
                TranslationCardOutput::Delete(_) | TranslationCardOutput::Update(_) => {}
            },
        }
    }
}

impl CatalogGridScreen {
    fn current_item(&self, item: &StoreItem) -> StoreItem {
        for shelf in &self.shelves {
            if let Some(found) = shelf.items.iter().find(|i| i.id == item.id) {
                return found.clone();
            }
        }
        item.clone()
    }

    fn language_options(&self) -> Vec<(String, String)> {
        let mut map: BTreeMap<String, String> = BTreeMap::new();
        for item in &self.initial_items {
            let resolved = self.current_item(item);
            let display = item.language_display();
            let label = if display.trim().is_empty() {
                resolved.language.clone()
            } else {
                display
            };
            map.entry(resolved.language).or_insert(label);
        }

        let mut opts: Vec<(String, String)> = map.into_iter().collect();
        opts.sort_by(|a, b| a.1.to_lowercase().cmp(&b.1.to_lowercase()));
        opts
    }

    fn filtered_items(&self) -> Vec<StoreItem> {
        let q = self.search_query.trim().to_lowercase();

        let mut items: Vec<StoreItem> = self
            .initial_items
            .iter()
            .map(|i| self.current_item(i))
            .filter(|item| {
                if let Some(ref lang) = self.filter_language
                    && &item.language != lang
                {
                    return false;
                }
                if !q.is_empty() {
                    let matches_name = item.name.to_lowercase().contains(&q);
                    let matches_id = item.id.to_lowercase().contains(&q);
                    return matches_name || matches_id;
                }
                true
            })
            .collect();

        match self.sort_order {
            CatalogSortOrder::Name => {
                items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            }
            CatalogSortOrder::Language => {
                items.sort_by(|a, b| {
                    a.language_display()
                        .to_lowercase()
                        .cmp(&b.language_display().to_lowercase())
                });
            }
        }

        items
    }

    fn card_status(&self, item: &StoreItem) -> TranslationCardStatus {
        if let Some(ref install) = self.active_install
            && install.item_id == item.id
        {
            if let Some(ref failed) = install.failed {
                return TranslationCardStatus::Failed(failed.clone());
            }
            if install.done {
                return TranslationCardStatus::Installed;
            }
            if install.fraction <= 0.0 {
                return TranslationCardStatus::Pending;
            }
            return TranslationCardStatus::Installing {
                fraction: install.fraction,
            };
        }

        if item.installed {
            TranslationCardStatus::Installed
        } else {
            TranslationCardStatus::Idle
        }
    }

    fn subtitle_text(&self) -> String {
        if let Some(ref install) = self.active_install
            && !install.done
            && install.failed.is_none()
        {
            let pct = (install.fraction.clamp(0.0, 1.0) * 100.0).round() as i32;
            let name = self
                .initial_items
                .iter()
                .find(|i| i.id == install.item_id)
                .map(|i| i.name.as_str())
                .unwrap_or(&install.item_id);
            return gettext("Installing {}… {}%")
                .replacen("{}", name, 1)
                .replacen("{}", &pct.to_string(), 1);
        }
        self.subtitle.clone().unwrap_or_default()
    }

    fn update_header_subtitle(&self, widget: &adw::WindowTitle) {
        widget.set_subtitle(&self.subtitle_text());
    }

    fn sync_or_rebuild(&mut self, flow_box: &gtk::FlowBox, sender: &ComponentSender<Self>) {
        let items = self.filtered_items();
        let same_order: Vec<&str> = items.iter().map(|i| i.id.as_str()).collect();
        let existing_order: Vec<&str> = self.cards.keys().map(String::as_str).collect();

        if same_order.len() == existing_order.len()
            && items.iter().all(|i| self.cards.contains_key(&i.id))
        {
            for item in items {
                if let Some(card) = self.cards.get(&item.id) {
                    card.emit(TranslationCardInput::SetStatus(self.card_status(&item)));
                }
            }
        } else {
            self.rebuild_grid(flow_box, sender);
        }
    }

    fn rebuild_grid(&mut self, flow_box: &gtk::FlowBox, sender: &ComponentSender<Self>) {
        while let Some(child) = flow_box.first_child() {
            flow_box.remove(&child);
        }
        self.cards.clear();

        let items = self.filtered_items();
        for item in items {
            let status = self.card_status(&item);
            let card = TranslationCard::builder()
                .launch(TranslationCardInit {
                    item: item.clone(),
                    status,
                    can_delete: false,
                    width: 150,
                })
                .forward(sender.input_sender(), CatalogGridInput::CardEvent);

            flow_box.insert(card.widget(), -1);
            self.cards.insert(item.id.clone(), card);
        }
    }
}

fn attach_sort_popover(
    button: &gtk::MenuButton,
    current: CatalogSortOrder,
    sender: &ComponentSender<CatalogGridScreen>,
) {
    let popover = gtk::Popover::new();
    let box_layout = gtk::Box::new(gtk::Orientation::Vertical, 4);
    box_layout.set_margin_all(8);

    let header = gtk::Label::builder()
        .label(&gettext("Sort By"))
        .halign(gtk::Align::Start)
        .margin_start(6)
        .margin_bottom(4)
        .build();
    header.add_css_class("heading");
    box_layout.append(&header);

    for order in [CatalogSortOrder::Name, CatalogSortOrder::Language] {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let lbl = gtk::Label::builder()
            .label(&order.label())
            .halign(gtk::Align::Start)
            .hexpand(true)
            .build();
        row.append(&lbl);

        if current == order {
            row.append(&gtk::Image::from_icon_name("object-select-symbolic"));
        }

        let btn = gtk::Button::new();
        btn.set_child(Some(&row));
        btn.add_css_class("flat");

        let pop_weak = popover.downgrade();
        let sender = sender.clone();
        btn.connect_clicked(move |_| {
            sender.input(CatalogGridInput::SetSortOrder(order));
            if let Some(pop) = pop_weak.upgrade() {
                pop.popdown();
            }
        });

        box_layout.append(&btn);
    }

    popover.set_child(Some(&box_layout));
    button.set_popover(Some(&popover));
}

fn attach_filter_popover(
    button: &gtk::MenuButton,
    options: &[(String, String)],
    selected_code: Option<&str>,
    sender: &ComponentSender<CatalogGridScreen>,
) {
    let popover = gtk::Popover::new();
    let box_layout = gtk::Box::new(gtk::Orientation::Vertical, 4);
    box_layout.set_margin_all(8);
    box_layout.set_size_request(220, -1);

    let header = gtk::Label::builder()
        .label(&gettext("Filter by Language"))
        .halign(gtk::Align::Start)
        .margin_start(6)
        .margin_bottom(4)
        .build();
    header.add_css_class("heading");
    box_layout.append(&header);

    // "All Languages" row
    {
        let is_selected = selected_code.is_none();
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let lbl = gtk::Label::builder()
            .label(&gettext("All Languages"))
            .halign(gtk::Align::Start)
            .hexpand(true)
            .build();
        row.append(&lbl);
        if is_selected {
            row.append(&gtk::Image::from_icon_name("object-select-symbolic"));
        }

        let btn = gtk::Button::new();
        btn.set_child(Some(&row));
        btn.add_css_class("flat");

        let pop_weak = popover.downgrade();
        let sender = sender.clone();
        btn.connect_clicked(move |_| {
            sender.input(CatalogGridInput::SetFilterLanguage(None));
            if let Some(pop) = pop_weak.upgrade() {
                pop.popdown();
            }
        });
        box_layout.append(&btn);
    }

    box_layout.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    let list_scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .max_content_height(260)
        .propagate_natural_height(true)
        .build();

    let inner_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    for (code, label) in options {
        let is_selected = selected_code == Some(code.as_str());
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let lbl = gtk::Label::builder()
            .label(label)
            .halign(gtk::Align::Start)
            .hexpand(true)
            .ellipsize(gtk::pango::EllipsizeMode::End)
            .build();
        row.append(&lbl);
        if is_selected {
            row.append(&gtk::Image::from_icon_name("object-select-symbolic"));
        }

        let btn = gtk::Button::new();
        btn.set_child(Some(&row));
        btn.add_css_class("flat");

        let code_owned = code.clone();
        let pop_weak = popover.downgrade();
        let sender = sender.clone();
        btn.connect_clicked(move |_| {
            sender.input(CatalogGridInput::SetFilterLanguage(Some(
                code_owned.clone(),
            )));
            if let Some(pop) = pop_weak.upgrade() {
                pop.popdown();
            }
        });
        inner_box.append(&btn);
    }

    list_scroll.set_child(Some(&inner_box));
    box_layout.append(&list_scroll);

    popover.set_child(Some(&box_layout));
    button.set_popover(Some(&popover));
}
