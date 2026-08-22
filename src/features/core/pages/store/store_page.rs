use std::collections::BTreeMap;
use std::sync::Arc;

use adw::prelude::*;
use relm4::factory::FactoryVecDeque;
use relm4::prelude::*;

use xbible_engine::engines::module_engine::sword_module::module::SwordModule;
use xbible_engine::engines::xbible_engine::engine::XBibleEngine;

use crate::features::core::components::swordmodule_section::{ModuleSection, ModuleSectionInit};

// ---------------------------------------------------------------------
// Small factory for the category pill row. Could be split into its own
// file the same way ModuleTile was.
// ---------------------------------------------------------------------

pub struct CategoryChip {
    pub name: String,
    pub is_selected: bool,
}

#[derive(Debug)]
pub enum CategoryChipInput {
    SetSelected(bool),
}

#[relm4::factory(pub)]
impl FactoryComponent for CategoryChip {
    type Init = String;
    type Input = CategoryChipInput;
    type Output = String;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Button {
            #[watch]
            set_label: &self.name,
            #[watch]
            set_class_active: ("suggested-action", self.is_selected),
            add_css_class: "pill",

            connect_clicked[sender, name = self.name.clone()] => move |_| {
                sender.output(name.clone());
            }
        }
    }

    fn init_model(name: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { name, is_selected: false }
    }

    fn update(&mut self, input: Self::Input, _sender: FactorySender<Self>) {
        match input {
            CategoryChipInput::SetSelected(v) => self.is_selected = v,
        }
    }
}

// ---------------------------------------------------------------------
// StorePage
// ---------------------------------------------------------------------

pub struct StorePage {
    engine: Arc<XBibleEngine>,

    remote_sources: Vec<String>,
    selected_source: Option<String>,

    all_modules: Vec<SwordModule>,
    categories: Vec<String>,
    selected_category: Option<String>,

    search_query: String,
    is_loading: bool,
    error: Option<String>,

    category_chips: FactoryVecDeque<CategoryChip>,

    // ModuleSection is a plain Component, not a FactoryComponent, so it
    // can't live inside a FactoryVecDeque. We manage its Controllers by
    // hand and mutate `sections_box`'s children directly instead.
    sections: Vec<Controller<ModuleSection>>,
    sections_box: gtk::Box,
}

#[derive(Debug)]
pub enum StorePageInput {
    SelectSource(String),
    SearchChanged(String),
    SelectCategory(String),
    Refresh,
    ModuleSectionOutput(String),
    DeleteModule(String),
}

#[derive(Clone, Debug)]
pub enum StorePageOutput {
    ToggleSidebar,
    ModuleSelected(String),
}

#[relm4::component(pub)]
impl Component for StorePage {
    type Init = Arc<XBibleEngine>;
    type Input = StorePageInput;
    type Output = StorePageOutput;
    type CommandOutput = ();

    view! {
        #[root]
        adw::NavigationPage {
            set_title: "Audio Bible",

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle { set_title: "Audio Bible" },
                    set_show_title: false,
                    add_css_class: "flat",

                    pack_start = &gtk::ToggleButton {
                        set_icon_name: "sidebar-show-symbolic",
                        connect_clicked[sender] => move |_| {
                            let _ = sender.output(StorePageOutput::ToggleSidebar);
                        }
                    },

                    pack_end = &gtk::DropDown {
                        #[watch]
                        set_model: Some(&gtk::StringList::new(
                            &model.remote_sources.iter().map(String::as_str).collect::<Vec<_>>()
                        )),
                        connect_selected_notify[sender] => move |dd| {
                            if let Some(obj) = dd.selected_item() {
                                if let Ok(s) = obj.downcast::<gtk::StringObject>() {
                                    sender.input(StorePageInput::SelectSource(s.string().to_string()));
                                }
                            }
                        }
                    }
                },

                #[wrap(Some)]
                set_content = &adw::Clamp {
                    set_maximum_size: 1500,
                    set_tightening_threshold: 1000,

                    #[wrap(Some)]
                    set_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 8,
                        set_margin_all: 12,

                        gtk::SearchEntry {
                            set_placeholder_text: Some("Search Bibles, commentaries..."),
                            connect_search_changed[sender] => move |entry| {
                                sender.input(StorePageInput::SearchChanged(entry.text().to_string()));
                            }
                        },

                        gtk::ScrolledWindow {
                            set_hscrollbar_policy: gtk::PolicyType::Automatic,
                            set_vscrollbar_policy: gtk::PolicyType::Never,
                            set_propagate_natural_height: true,

                            #[local_ref]
                            category_box -> gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 6,
                            }
                        },

                        gtk::Stack {
                            set_vexpand: true,
                            #[watch]
                            set_visible_child_name: if model.is_loading {
                                "loading"
                            } else if model.error.is_some() {
                                "error"
                            } else {
                                "content"
                            },

                            add_named[Some("loading")] = &adw::StatusPage {
                                set_title: "Loading store…",
                                set_icon_name: Some("content-loading-symbolic"),
                            },

                            add_named[Some("error")] = &adw::StatusPage {
                                #[watch]
                                set_description: model.error.as_deref(),
                                set_title: "Couldn't load the store",
                                set_icon_name: Some("dialog-error-symbolic"),
                            },

                            add_named[Some("content")] = &gtk::ScrolledWindow {
                                set_vexpand: true,

                                #[local_ref]
                                sections_box -> gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 12,
                                    set_margin_bottom: 24,
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn init(
        engine: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let category_chips = FactoryVecDeque::builder()
            .launch(gtk::Box::default())
            .forward(sender.input_sender(), StorePageInput::SelectCategory);

        let sections_box = gtk::Box::default();

        let model = StorePage {
            engine,
            remote_sources: Vec::new(),
            selected_source: None,
            all_modules: Vec::new(),
            categories: Vec::new(),
            selected_category: None,
            search_query: String::new(),
            is_loading: true,
            error: None,
            category_chips,
            sections: Vec::new(),
            sections_box,
        };

        let category_box = model.category_chips.widget();
        let sections_box = &model.sections_box;
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            StorePageInput::SelectSource(source) => {
                if self.selected_source.as_deref() == Some(source.as_str()) {
                    return;
                }
                self.selected_source = Some(source.clone());
                self.search_query.clear();
                self.is_loading = true;
                self.error = None;

                // Direct, synchronous, main-thread call.
                let modules = self.engine.fetch_remote_modules(&source);
                self.apply_loaded_modules(modules, &sender);
            }
            StorePageInput::Refresh => {
                if let Some(source) = self.selected_source.clone() {
                    self.is_loading = true;
                    self.error = None;

                    let modules = self.engine.fetch_remote_modules(&source);
                    self.apply_loaded_modules(modules, &sender);
                }
            }
            StorePageInput::SearchChanged(query) => {
                self.search_query = query;
                self.rebuild_sections(&sender);
            }
            StorePageInput::SelectCategory(category) => {
                self.selected_category = Some(category);
                self.sync_category_selection();
                self.rebuild_sections(&sender);
            }
            StorePageInput::ModuleSectionOutput(module_name) => {
                let _ = sender.output(StorePageOutput::ModuleSelected(module_name));
            }
            StorePageInput::DeleteModule(module_name) => {
                // Direct, synchronous, main-thread call — same as every
                // other engine call now. Adjust the method name below to
                // whatever XBibleEngine actually exposes for removal.
                //self.engine.remove_module(&module_name);

                self.all_modules.retain(|m| m.name != module_name);
                self.rebuild_sections(&sender);
            }
        }
    }
}

impl StorePage {
    fn apply_loaded_modules(&mut self, modules: Vec<SwordModule>, sender: &ComponentSender<Self>) {
        self.is_loading = false;

        if modules.is_empty() {
            self.error = Some("No modules were returned from this source.".to_string());
            self.all_modules.clear();
            self.categories.clear();
            self.rebuild_category_chips();
            self.rebuild_sections(sender);
            return;
        }

        self.error = None;
        self.all_modules = modules;

        let mut cats: Vec<String> =
            self.all_modules.iter().map(|m| m.category.clone()).collect();
        cats.sort();
        cats.dedup();
        self.categories = cats;

        if self
            .selected_category
            .as_ref()
            .map_or(true, |c| !self.categories.contains(c))
        {
            self.selected_category = self.categories.first().cloned();
        }

        self.rebuild_category_chips();
        self.rebuild_sections(sender);
    }

    fn rebuild_category_chips(&mut self) {
        let mut guard = self.category_chips.guard();
        guard.clear();
        for cat in &self.categories {
            guard.push_back(cat.clone());
        }
        drop(guard);
        self.sync_category_selection();
    }

    fn sync_category_selection(&mut self) {
        let selected = self.selected_category.clone();
        let mut guard = self.category_chips.guard();
        for i in 0..guard.len() {
            if let Some(chip) = guard.get_mut(i) {
                chip.is_selected = Some(chip.name.clone()) == selected;
            }
        }
    }

    fn rebuild_sections(&mut self, sender: &ComponentSender<Self>) {
        let query = self.search_query.to_lowercase();

        let filtered: Vec<&SwordModule> = self
            .all_modules
            .iter()
            .filter(|m| self.selected_category.as_deref() == Some(m.category.as_str()))
            .filter(|m| {
                query.is_empty()
                    || m.name.to_lowercase().contains(&query)
                    || m.description.to_lowercase().contains(&query)
                    || m.language.to_lowercase().contains(&query)
            })
            .collect();

        let mut by_language: BTreeMap<String, Vec<SwordModule>> = BTreeMap::new();
        for m in filtered {
            by_language.entry(m.language.clone()).or_default().push(m.clone());
        }

        while let Some(child) = self.sections_box.first_child() {
            self.sections_box.remove(&child);
        }
        self.sections.clear();

        for (language_name, modules) in by_language {
            let controller = ModuleSection::builder()
                .launch(ModuleSectionInit { language_name, modules })
                .forward(sender.input_sender(), StorePageInput::ModuleSectionOutput);

            self.sections_box.append(controller.widget());
            self.sections.push(controller);
        }
    }
}