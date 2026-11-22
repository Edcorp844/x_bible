use adw::prelude::*;
use gettextrs::gettext;
use relm4::{ComponentController, Controller, WorkerController, prelude::*};
use xbible_engine::features::store::{LanguageFacet, StoreItem, StoreShelf};

use crate::{
    core::engine::SharedEngine,
    features::store::store_components::{
        catalog_grid_view::{
            CatalogGridInit, CatalogGridInput, CatalogGridOutput, CatalogGridScreen,
        },
        item_detail_view::{
            StoreItemDetailInit, StoreItemDetailInput, StoreItemDetailOutput, StoreItemDetailView,
        },
        language_list_view::{
            LanguageListInit, LanguageListInput, LanguageListOutput, LanguageListView,
        },
        translation_card::{
            TranslationCard, TranslationCardInit, TranslationCardInput, TranslationCardOutput,
            TranslationCardStatus,
        },
    },
    workers::store_worker::{InstallState, StoreWorker, StoreWorkerInput, StoreWorkerOutput},
};

#[derive(Debug)]
pub(crate) struct StoreView {
    worker: WorkerController<StoreWorker>,
    shelves: Vec<StoreShelf>,
    languages: Vec<LanguageFacet>,
    search_query: String,
    is_loading: bool,
    error_message: Option<String>,
    active_install: Option<InstallState>,
    download_state: String,
    shelf_cards: Vec<(String, Controller<TranslationCard>)>,
    detail_page: Option<Controller<StoreItemDetailView>>,
    grid_page: Option<Controller<CatalogGridScreen>>,
    languages_page: Option<Controller<LanguageListView>>,
}

#[derive(Debug)]
pub enum StoreViewInput {
    SetSidebarCollapsed(bool),
    RefreshCatalog,
    SearchChanged(String),
    InstallItem(StoreItem),
    UninstallItem(StoreItem),
    OpenItem(StoreItem),
    SelectItem(StoreItem),
    OpenShelf(StoreShelf),
    SelectLanguage(LanguageFacet),
    OpenAllLanguages,
    CancelActiveInstall,
    DismissError,
    CardEvent(TranslationCardOutput),
    DetailEvent(StoreItemDetailOutput),
    GridEvent(CatalogGridOutput),
    LanguageListEvent(LanguageListOutput),
    WorkerEvent(StoreWorkerOutput),
}

#[derive(Debug)]
pub enum StoreViewOutput {
    ShowError(String),
    OpenItem(StoreItem),
}

#[relm4::component(pub)]
impl relm4::Component for StoreView {
    type Init = SharedEngine;
    type Input = StoreViewInput;
    type Output = StoreViewOutput;
    type CommandOutput = ();

    view! {
        adw::NavigationPage {
            #[wrap(Some)]
            #[name = "nav_view"]
            set_child = &adw::NavigationView {
                add = &adw::NavigationPage {
                    set_tag: Some("store-home"),
                    set_title: &gettext("Bible Store"),

                    #[wrap(Some)]
                    set_child = &adw::ToolbarView {
                        add_top_bar = &adw::HeaderBar {
                            #[wrap(Some)]
                            #[name = "header_title_widget"]
                            set_title_widget = &adw::WindowTitle {
                                set_title: &gettext("Bible Store"),
                                set_subtitle: &model.download_state,
                            },

                            pack_end = &gtk::Button {
                                set_icon_name: "view-refresh-symbolic",
                                set_tooltip_text: Some(&gettext("Refresh Catalog")),
                                connect_clicked[sender] => move |_| {
                                    sender.input(StoreViewInput::RefreshCatalog);
                                },
                            },

                            #[name = "search_entry"]
                            pack_end = &gtk::SearchEntry {
                                set_placeholder_text: Some(&gettext("Search translations, people, places…")),
                                set_hexpand: true,
                                connect_search_changed[sender] => move |entry| {
                                    sender.input(StoreViewInput::SearchChanged(entry.text().to_string()));
                                },
                            },
                        },

                        #[wrap(Some)]
                        set_content = &gtk::ScrolledWindow {
                            set_hscrollbar_policy: gtk::PolicyType::Never,
                            set_vexpand: true,

                            #[name = "main_stack_box"]
                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 20,
                                set_margin_top: 12,
                                set_margin_bottom: 24,

                                // Active install / failure banner
                                #[name = "install_banner_box"]
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Horizontal,
                                    set_spacing: 12,
                                    set_margin_start: 16,
                                    set_margin_end: 16,
                                    set_visible: false,
                                    add_css_class: "card",
                                },

                                // Store unavailable banner / empty state
                                #[name = "error_box"]
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 8,
                                    set_margin_start: 24,
                                    set_margin_end: 24,
                                    set_margin_top: 32,
                                    set_visible: false,
                                },

                                // Languages quick chips section
                                #[name = "languages_section"]
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 16,
                                    set_visible: false,

                                    // Languages Header
                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Horizontal,
                                        set_spacing: 10,
                                        set_margin_start: 16,
                                        set_margin_end: 16,
                                        set_halign: gtk::Align::Start,
                                        set_cursor_from_name: Some("pointer"),

                                        gtk::Label {
                                            set_label: &gettext("Languages"),
                                            add_css_class: "title-3",
                                            set_halign: gtk::Align::Start,
                                        },

                                        gtk::Image {
                                            set_icon_name: Some("go-next-symbolic"),
                                            set_halign: gtk::Align::Start,
                                        },

                                        add_controller = gtk::GestureClick {
                                            connect_pressed[sender] => move |gesture, _, _, _| {
                                                gesture.set_state(gtk::EventSequenceState::Claimed);
                                                sender.input(StoreViewInput::OpenAllLanguages);
                                            }
                                        }
                                    },

                                    #[name = "language_chips_flow"]
                                    adw::WrapBox {
                                        set_margin_start: 16,
                                        set_margin_end: 16,
                                        set_line_spacing: 16,
                                        set_child_spacing: 16,
                                    },
                                },

                                // Loading / empty search / catalog shelves container
                                #[name = "shelves_container"]
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 24,
                                },
                            },
                        },
                    },
                },
            },
        }
    }

    fn init(
        engine: Self::Init,
        _root: Self::Root,
        sender: relm4::prelude::ComponentSender<Self>,
    ) -> relm4::prelude::ComponentParts<Self> {
        let worker = StoreWorker::builder()
            .detach_worker(engine)
            .forward(sender.input_sender(), StoreViewInput::WorkerEvent);

        let mut model = Self {
            worker,
            shelves: Vec::new(),
            languages: Vec::new(),
            search_query: String::new(),
            is_loading: false,
            error_message: None,
            active_install: None,
            download_state: String::new(),
            shelf_cards: Vec::new(),
            detail_page: None,
            grid_page: None,
            languages_page: None,
        };

        let widgets = view_output!();
        model.refresh_view(&widgets, &sender);

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
            StoreViewInput::SetSidebarCollapsed(_) => {}

            StoreViewInput::RefreshCatalog => {
                self.worker.emit(StoreWorkerInput::RefreshCatalog);
            }

            StoreViewInput::SearchChanged(query) => {
                self.search_query = query;
                self.rebuild_shelves(&widgets.shelves_container, &sender);
            }

            StoreViewInput::InstallItem(item) => {
                self.worker.emit(StoreWorkerInput::InstallItem(item));
            }

            StoreViewInput::UninstallItem(item) => {
                self.worker.emit(StoreWorkerInput::UninstallItem {
                    id: item.id,
                    kind: item.kind,
                });
            }

            StoreViewInput::OpenItem(item) => {
                let _ = sender.output(StoreViewOutput::OpenItem(item));
            }

            StoreViewInput::SelectItem(item) => {
                let detail = StoreItemDetailView::builder()
                    .launch(StoreItemDetailInit {
                        item,
                        shelves: self.shelves.clone(),
                        active_install: self.active_install.clone(),
                    })
                    .forward(sender.input_sender(), StoreViewInput::DetailEvent);

                widgets.nav_view.push(detail.widget());
                self.detail_page = Some(detail);
            }

            StoreViewInput::OpenShelf(shelf) => {
                let grid = CatalogGridScreen::builder()
                    .launch(CatalogGridInit {
                        title: shelf.title,
                        subtitle: shelf.subtitle,
                        initial_items: shelf.items,
                        shelves: self.shelves.clone(),
                        active_install: self.active_install.clone(),
                    })
                    .forward(sender.input_sender(), StoreViewInput::GridEvent);

                widgets.nav_view.push(grid.widget());
                self.grid_page = Some(grid);
            }

            StoreViewInput::SelectLanguage(language) => {
                let items = deduplicated_items(
                    self.shelves
                        .iter()
                        .flat_map(|s| s.items.iter())
                        .filter(|item| item.language == language.code)
                        .cloned()
                        .collect(),
                );
                let trimmed_eng = language.english_name.trim();
                let subtitle = if trimmed_eng.is_empty() {
                    None
                } else {
                    Some(trimmed_eng.to_string())
                };

                let grid = CatalogGridScreen::builder()
                    .launch(CatalogGridInit {
                        title: language.name,
                        subtitle,
                        initial_items: items,
                        shelves: self.shelves.clone(),
                        active_install: self.active_install.clone(),
                    })
                    .forward(sender.input_sender(), StoreViewInput::GridEvent);

                widgets.nav_view.push(grid.widget());
                self.grid_page = Some(grid);
            }

            StoreViewInput::OpenAllLanguages => {
                let lang_view = LanguageListView::builder()
                    .launch(LanguageListInit {
                        languages: self.languages.clone(),
                    })
                    .forward(sender.input_sender(), StoreViewInput::LanguageListEvent);

                widgets.nav_view.push(lang_view.widget());
                self.languages_page = Some(lang_view);
            }

            StoreViewInput::CancelActiveInstall => {
                self.worker.emit(StoreWorkerInput::CancelActiveInstall);
            }

            StoreViewInput::DismissError => {
                self.error_message = None;
                self.refresh_view(widgets, &sender);
            }

            StoreViewInput::CardEvent(event) => match event {
                TranslationCardOutput::Install(item) => {
                    self.worker.emit(StoreWorkerInput::InstallItem(item));
                }
                TranslationCardOutput::Open(item) => {
                    let _ = sender.output(StoreViewOutput::OpenItem(item));
                }
                TranslationCardOutput::Selected(item) => {
                    sender.input(StoreViewInput::SelectItem(item));
                }
                TranslationCardOutput::Delete(item) => {
                    self.worker.emit(StoreWorkerInput::UninstallItem {
                        id: item.id,
                        kind: item.kind,
                    });
                }
                TranslationCardOutput::Update(item) => {
                    self.worker.emit(StoreWorkerInput::InstallItem(item));
                }
            },

            StoreViewInput::DetailEvent(event) => match event {
                StoreItemDetailOutput::InstallItem(item) => {
                    self.worker.emit(StoreWorkerInput::InstallItem(item));
                }
                StoreItemDetailOutput::OpenItem(item) => {
                    let _ = sender.output(StoreViewOutput::OpenItem(item));
                }
                StoreItemDetailOutput::UninstallItem(item) => {
                    self.worker.emit(StoreWorkerInput::UninstallItem {
                        id: item.id,
                        kind: item.kind,
                    });
                }
            },

            StoreViewInput::GridEvent(event) => match event {
                CatalogGridOutput::InstallItem(item) => {
                    self.worker.emit(StoreWorkerInput::InstallItem(item));
                }
                CatalogGridOutput::OpenItem(item) => {
                    let _ = sender.output(StoreViewOutput::OpenItem(item));
                }
                CatalogGridOutput::SelectItem(item) => {
                    sender.input(StoreViewInput::SelectItem(item));
                }
            },

            StoreViewInput::LanguageListEvent(event) => match event {
                LanguageListOutput::Selected(lang) => {
                    sender.input(StoreViewInput::SelectLanguage(lang));
                }
            },

            StoreViewInput::WorkerEvent(output) => match output {
                StoreWorkerOutput::LoadingChanged(loading) => {
                    self.is_loading = loading;
                    widgets
                        .header_title_widget
                        .set_subtitle(&self.download_state);
                    self.refresh_error_banner(&widgets.error_box, &sender);
                    if self.is_loading && self.shelves.is_empty() {
                        self.rebuild_shelves(&widgets.shelves_container, &sender);
                    }
                }
                StoreWorkerOutput::CatalogUpdated { shelves, languages } => {
                    self.error_message = None;
                    self.shelves = shelves;
                    self.languages = languages;

                    if let Some(ref lang_page) = self.languages_page {
                        lang_page.emit(LanguageListInput::SetLanguages(self.languages.clone()));
                    }
                    self.sync_subpages();
                    self.refresh_view(widgets, &sender);
                }
                StoreWorkerOutput::ActiveInstallChanged(install) => {
                    self.active_install = install;
                    self.sync_download_state();
                    widgets
                        .header_title_widget
                        .set_subtitle(&self.download_state);
                    self.sync_subpages();
                    self.refresh_install_banner(&widgets.install_banner_box, &sender);
                    self.sync_card_statuses();
                }
                StoreWorkerOutput::Error(err) => {
                    if self.shelves.is_empty() {
                        self.error_message = Some(err.clone());
                    }
                    let _ = sender.output(StoreViewOutput::ShowError(err));
                    self.refresh_error_banner(&widgets.error_box, &sender);
                }
                StoreWorkerOutput::InstalledStatus { .. } => {}
            },
        }
    }
}

impl StoreView {
    fn sync_download_state(&mut self) {
        if let Some(ref install) = self.active_install
            && !install.done
            && install.failed.is_none()
        {
            let pct = (install.fraction.clamp(0.0, 1.0) * 100.0).round() as i32;
            let name = self.display_name_for(install);
            self.download_state = gettext("Installing {}… {}%")
                .replacen("{}", &name, 1)
                .replacen("{}", &pct.to_string(), 1);
        } else {
            self.download_state.clear();
        }
    }

    fn display_name_for(&self, state: &InstallState) -> String {
        for shelf in &self.shelves {
            if let Some(item) = shelf.items.iter().find(|i| i.id == state.item_id) {
                return item.name.clone();
            }
        }
        state.item_id.clone()
    }

    fn filtered_shelves(&self) -> Vec<StoreShelf> {
        let query = self.search_query.trim().to_lowercase();
        if query.is_empty() {
            return self.shelves.clone();
        }

        self.shelves
            .iter()
            .filter_map(|shelf| {
                let shelf_matches = shelf.title.to_lowercase().contains(&query);
                let items: Vec<StoreItem> = shelf
                    .items
                    .iter()
                    .filter(|item| {
                        let lang_name_match = item
                            .language_name
                            .as_deref()
                            .map(|n| n.to_lowercase().contains(&query))
                            .unwrap_or(false);
                        shelf_matches
                            || item.name.to_lowercase().contains(&query)
                            || item.id.to_lowercase().contains(&query)
                            || item.language.to_lowercase().contains(&query)
                            || lang_name_match
                    })
                    .cloned()
                    .collect();

                if items.is_empty() {
                    None
                } else {
                    Some(StoreShelf {
                        title: shelf.title.clone(),
                        subtitle: shelf.subtitle.clone(),
                        items,
                    })
                }
            })
            .collect()
    }

    fn card_status_for(&self, item: &StoreItem) -> TranslationCardStatus {
        if let Some(ref install) = self.active_install
            && install.item_id == item.id
        {
            if let Some(ref err) = install.failed {
                return TranslationCardStatus::Failed(err.clone());
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

    fn item_by_id(&self, id: &str) -> Option<&StoreItem> {
        self.shelves
            .iter()
            .flat_map(|s| s.items.iter())
            .find(|item| item.id == id)
    }

    fn sync_subpages(&self) {
        if let Some(ref detail) = self.detail_page {
            detail.emit(StoreItemDetailInput::SyncState {
                shelves: self.shelves.clone(),
                active_install: self.active_install.clone(),
            });
        }
        if let Some(ref grid) = self.grid_page {
            grid.emit(CatalogGridInput::SyncState {
                shelves: self.shelves.clone(),
                active_install: self.active_install.clone(),
            });
        }
    }

    fn sync_card_statuses(&self) {
        for (item_id, card) in &self.shelf_cards {
            if let Some(item) = self.item_by_id(item_id) {
                card.emit(TranslationCardInput::SetStatus(self.card_status_for(item)));
            }
        }
    }

    fn refresh_view(
        &mut self,
        widgets: &<Self as Component>::Widgets,
        sender: &ComponentSender<Self>,
    ) {
        widgets
            .header_title_widget
            .set_subtitle(&self.download_state);
        self.refresh_install_banner(&widgets.install_banner_box, sender);
        self.refresh_error_banner(&widgets.error_box, sender);
        self.refresh_language_chips(
            &widgets.languages_section,
            &widgets.language_chips_flow,
            sender,
        );
        self.rebuild_shelves(&widgets.shelves_container, sender);
    }

    fn refresh_install_banner(&self, banner: &gtk::Box, sender: &ComponentSender<Self>) {
        while let Some(child) = banner.first_child() {
            banner.remove(&child);
        }

        let Some(ref install) = self.active_install else {
            banner.set_visible(false);
            return;
        };

        if install.done {
            banner.set_visible(false);
            return;
        }

        banner.set_visible(true);
        banner.set_margin_top(4);
        banner.set_margin_bottom(4);

        let inner = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        inner.set_margin_all(12);

        if let Some(ref failed) = install.failed {
            let icon = gtk::Image::from_icon_name("dialog-warning-symbolic");
            icon.add_css_class("error");
            inner.append(&icon);

            let text_col = gtk::Box::new(gtk::Orientation::Vertical, 2);
            text_col.set_hexpand(true);

            let title = gtk::Label::builder()
                .label(
                    &gettext("Install Failed — {}").replace("{}", &self.display_name_for(install)),
                )
                .halign(gtk::Align::Start)
                .build();
            title.add_css_class("heading");

            let err_lbl = gtk::Label::builder()
                .label(failed)
                .halign(gtk::Align::Start)
                .wrap(true)
                .lines(2)
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .build();
            err_lbl.add_css_class("caption");
            err_lbl.add_css_class("dim-label");

            text_col.append(&title);
            text_col.append(&err_lbl);
            inner.append(&text_col);

            let dismiss_btn = gtk::Button::with_label(&gettext("Dismiss"));
            dismiss_btn.add_css_class("flat");
            dismiss_btn.connect_clicked({
                let sender = sender.clone();
                move |_| sender.input(StoreViewInput::CancelActiveInstall)
            });
            inner.append(&dismiss_btn);
        } else {
            let spinner = gtk::Spinner::builder().spinning(true).build();
            inner.append(&spinner);

            let text_col = gtk::Box::new(gtk::Orientation::Vertical, 2);
            text_col.set_hexpand(true);

            let title_lbl = gtk::Label::builder()
                .label(&self.download_state)
                .halign(gtk::Align::Start)
                .build();
            title_lbl.add_css_class("heading");
            text_col.append(&title_lbl);

            if !install.message.is_empty() {
                let msg_lbl = gtk::Label::builder()
                    .label(&install.message)
                    .halign(gtk::Align::Start)
                    .ellipsize(gtk::pango::EllipsizeMode::End)
                    .build();
                msg_lbl.add_css_class("caption");
                msg_lbl.add_css_class("dim-label");
                text_col.append(&msg_lbl);
            }

            inner.append(&text_col);

            let cancel_btn = gtk::Button::with_label(&gettext("Cancel"));
            cancel_btn.add_css_class("flat");
            cancel_btn.connect_clicked({
                let sender = sender.clone();
                move |_| sender.input(StoreViewInput::CancelActiveInstall)
            });
            inner.append(&cancel_btn);
        }

        banner.append(&inner);
    }

    fn refresh_error_banner(&self, error_box: &gtk::Box, sender: &ComponentSender<Self>) {
        while let Some(child) = error_box.first_child() {
            error_box.remove(&child);
        }

        if let Some(ref err) = self.error_message
            && self.shelves.is_empty()
        {
            error_box.set_visible(true);
            let status = adw::StatusPage::builder()
                .icon_name("network-error-symbolic")
                .title(&gettext("Store Unavailable"))
                .description(err)
                .build();

            let retry_btn = gtk::Button::with_label(&gettext("Retry"));
            retry_btn.add_css_class("pill");
            retry_btn.add_css_class("suggested-action");
            retry_btn.set_halign(gtk::Align::Center);
            retry_btn.connect_clicked({
                let sender = sender.clone();
                move |_| sender.input(StoreViewInput::RefreshCatalog)
            });
            status.set_child(Some(&retry_btn));
            error_box.append(&status);
        } else {
            error_box.set_visible(false);
        }
    }

    fn refresh_language_chips(
        &self,
        section: &gtk::Box,
        chips_box: &adw::WrapBox,
        sender: &ComponentSender<Self>,
    ) {
        while let Some(child) = chips_box.first_child() {
            chips_box.remove(&child);
        }

        if self.languages.is_empty() {
            section.set_visible(false);
            return;
        }

        section.set_visible(true);
        for lang in self.languages.iter().take(10) {
            let chip = gtk::Button::with_label(&lang.name);
            chip.add_css_class("pill");
            let lang_clone = lang.clone();
            let sender = sender.clone();
            chip.connect_clicked(move |_| {
                sender.input(StoreViewInput::SelectLanguage(lang_clone.clone()));
            });
            chips_box.append(&chip);
        }
    }

    fn rebuild_shelves(&mut self, container: &gtk::Box, sender: &ComponentSender<Self>) {
        // Detach old widgets from container before dropping previous card controllers
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }
        let mut next_cards: Vec<(String, Controller<TranslationCard>)> = Vec::new();

        if self.is_loading && self.shelves.is_empty() {
            self.shelf_cards.clear();
            let spinner = gtk::Spinner::builder()
                .spinning(true)
                .width_request(32)
                .height_request(32)
                .halign(gtk::Align::Center)
                .margin_top(48)
                .build();
            container.append(&spinner);
            return;
        }

        let filtered = self.filtered_shelves();
        if filtered.is_empty() && !self.search_query.trim().is_empty() {
            self.shelf_cards.clear();
            let empty = adw::StatusPage::builder()
                .icon_name("system-search-symbolic")
                .title(&gettext("No Results"))
                .description(
                    &gettext("No store items match \"{}\".")
                        .replace("{}", self.search_query.trim()),
                )
                .build();
            container.append(&empty);
            return;
        }

        for shelf in filtered {
            let shelf_col = gtk::Box::new(gtk::Orientation::Vertical, 10);

            let header_col = gtk::Box::new(gtk::Orientation::Vertical, 4);
            header_col.set_margin_start(16);
            header_col.set_margin_end(16);

            let title_row = gtk::Box::new(gtk::Orientation::Horizontal, 10);
            title_row.set_halign(gtk::Align::Start);
            title_row.set_cursor_from_name(Some("pointer"));

            let title_lbl = gtk::Label::builder()
                .label(&shelf.title)
                .halign(gtk::Align::Start)
                .build();
            title_lbl.add_css_class("title-3");

            let chevron = gtk::Image::from_icon_name("go-next-symbolic");
            chevron.set_halign(gtk::Align::Start);
            chevron.add_css_class("dim-label");

            title_row.append(&title_lbl);
            title_row.append(&chevron);

            let click_gesture = gtk::GestureClick::new();
            let shelf_clone = shelf.clone();
            let sender_click = sender.clone();
            click_gesture.connect_pressed(move |gesture, _, _, _| {
                gesture.set_state(gtk::EventSequenceState::Claimed);
                sender_click.input(StoreViewInput::OpenShelf(shelf_clone.clone()));
            });
            title_row.add_controller(click_gesture);

            header_col.append(&title_row);

            if let Some(ref sub) = shelf.subtitle
                && !sub.trim().is_empty()
            {
                let sub_lbl = gtk::Label::builder()
                    .label(sub)
                    .halign(gtk::Align::Start)
                    .wrap(true)
                    .build();
                sub_lbl.add_css_class("caption");
                sub_lbl.add_css_class("dim-label");
                header_col.append(&sub_lbl);
            }

            shelf_col.append(&header_col);

            let row_scroll = gtk::ScrolledWindow::builder()
                .hscrollbar_policy(gtk::PolicyType::External)
                .vscrollbar_policy(gtk::PolicyType::Never)
                .build();

            let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            row_box.set_margin_start(16);
            row_box.set_margin_end(16);

            for item in shelf.items {
                let card = TranslationCard::builder()
                    .launch(TranslationCardInit {
                        item: item.clone(),
                        status: self.card_status_for(&item),
                        can_delete: false,
                        width: 150,
                    })
                    .forward(sender.input_sender(), StoreViewInput::CardEvent);

                row_box.append(card.widget());
                next_cards.push((item.id.clone(), card));
            }

            row_scroll.set_child(Some(&row_box));
            shelf_col.append(&row_scroll);
            container.append(&shelf_col);
        }

        // Replace old controllers only after new widgets are built and attached
        self.shelf_cards = next_cards;
    }
}

fn deduplicated_items(items: Vec<StoreItem>) -> Vec<StoreItem> {
    let mut seen = std::collections::HashSet::new();
    items
        .into_iter()
        .filter(|item| seen.insert(item.id.clone()))
        .collect()
}
