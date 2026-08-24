use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use adw::prelude::*;
use relm4::prelude::*;

use xbible_engine::engines::module_engine::sword_module::module::SwordModule;
use xbible_engine::engines::xbible_engine::engine::XBibleEngine;
use xbible_engine::engines::xbible_engine::xbible_engine_extensions::xbible_engine_task_ext::TaskState;

use crate::features::core::components::swordmodule::{
    ModuleMenuAction, ModuleTileInput, ModuleTileOutput,
};
use crate::features::core::components::swordmodule_section::{
    ModuleSection, ModuleSectionInit, ModuleSectionInput, ModuleSectionOutput,
};

// ---------------------------------------------------------------------
// Installation Status Enum
// ---------------------------------------------------------------------
#[derive(Debug, Clone, PartialEq)]
pub enum InstallationStatus {
    Idle,
    Pending,
    Installing(f64),
    Installed,
    Cancelled,
    Failed(String),
}

// ---------------------------------------------------------------------
// Category Chip Component
// ---------------------------------------------------------------------

pub struct CategoryChipInit {
    pub name: String,
    pub is_selected: bool,
}

pub struct CategoryChip {
    pub name: String,
    pub is_selected: bool,
}

#[derive(Debug)]
pub enum CategoryChipInput {
    SetSelected(bool),
}

#[derive(Debug, Clone)]
pub enum CategoryChipOutput {
    Clicked(String),
}

#[relm4::component(pub)]
impl Component for CategoryChip {
    type Init = CategoryChipInit;
    type Input = CategoryChipInput;
    type Output = CategoryChipOutput;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Button {
            #[watch]
            set_label: &model.name,
            #[watch]
            set_class_active: ("flat", !model.is_selected),

            connect_clicked[sender, name = model.name.clone()] => move |_| {
                let _ = sender.output(CategoryChipOutput::Clicked(name.clone()));
            }
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = CategoryChip {
            name: init.name,
            is_selected: init.is_selected,
        };

        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match input {
            CategoryChipInput::SetSelected(v) => self.is_selected = v,
        }
    }
}

// ---------------------------------------------------------------------
// StorePage Component
// ---------------------------------------------------------------------

pub struct StorePage {
    engine: Arc<XBibleEngine>,

    remote_sources: Vec<String>,
    selected_source: Option<String>,

    remote_modules: Vec<SwordModule>,
    categories: Vec<String>,
    selected_category: Option<String>,
    installation_states: BTreeMap<String, InstallationStatus>,
    active_tasks: BTreeMap<String, String>,

    search_query: String,
    is_loading: bool,
    error: Option<String>,

    category_chips: Vec<Controller<CategoryChip>>,
    category_box: adw::WrapBox,
    sections: Vec<Controller<ModuleSection>>,
    sections_box: gtk::Box,
    sources_dropdown: gtk::DropDown,
}

#[derive(Debug)]
pub enum StorePageInput {
    InitLoad,
    SourcesLoaded(Vec<String>),
    SelectSource(String),
    ModulesLoaded(Result<Vec<SwordModule>, String>),
    SearchChanged(String),
    SelectCategory(String),
    RefreshStore,
    InstallModule(SwordModule),
    TaskStarted {
        module_name: String,
        task_id: String,
    },
    UpdateInstallationStatus {
        module_name: String,
        status: InstallationStatus,
    },
    CancelInstall(String),
    HandleSectionOutput(ModuleSectionOutput),
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
            set_title: "Store",

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    set_title_widget: Some(&adw::WindowTitle::new("Store", "")),
                    set_show_title: false,
                    add_css_class: "flat",

                    pack_start = &gtk::ToggleButton {
                        set_icon_name: "sidebar-show-symbolic",
                        connect_clicked[sender] => move |_| {
                            let _ = sender.output(StorePageOutput::ToggleSidebar);
                        }
                    },

                    #[local_ref]
                    pack_end = sources_dropdown -> gtk::DropDown {}
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

                        #[local_ref]
                        category_box -> adw::WrapBox {
                            set_child_spacing: 6,
                            set_line_spacing: 6,
                        },

                        gtk::Stack {
                            set_vexpand: true,
                            #[watch]
                            set_visible_child_name: if model.is_loading && model.remote_modules.is_empty() {
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
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        log::info!("[StorePage] Initializing StorePage component...");

        let category_box = adw::WrapBox::default();
        let sections_box = gtk::Box::default();
        let sources_dropdown = gtk::DropDown::default();

        let input_sender = sender.input_sender().clone();
        sources_dropdown.connect_selected_item_notify(move |dd| {
            if let Some(obj) = dd.selected_item() {
                if let Ok(s) = obj.downcast::<gtk::StringObject>() {
                    let selected_str = s.string().to_string();
                    input_sender
                        .send(StorePageInput::SelectSource(selected_str))
                        .ok();
                }
            }
        });

        let model = StorePage {
            engine,
            remote_sources: Vec::new(),
            selected_source: None,
            remote_modules: Vec::new(),
            categories: Vec::new(),
            selected_category: None,
            installation_states: BTreeMap::new(),
            active_tasks: BTreeMap::new(),
            search_query: String::new(),
            is_loading: true,
            error: None,
            category_chips: Vec::new(),
            category_box,
            sections: Vec::new(),
            sections_box,
            sources_dropdown,
        };

        let category_box = &model.category_box;
        let sections_box = &model.sections_box;
        let sources_dropdown = &model.sources_dropdown;
        let widgets = view_output!();

        log::info!("[StorePage] Triggering StorePageInput::InitLoad...");
        sender.input(StorePageInput::InitLoad);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            StorePageInput::InitLoad => {
                self.is_loading = true;
                let engine_worker = self.engine.clone();
                let input_sender = sender.input_sender().clone();

                thread::spawn(move || {
                    let raw_sources = engine_worker.get_remote_sources();
                    let sources: Vec<String> = raw_sources
                        .into_iter()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();

                    if sources.is_empty() {
                        input_sender
                            .send(StorePageInput::ModulesLoaded(Err(
                                "No valid remote sources found.".to_string(),
                            )))
                            .ok();
                        return;
                    }

                    input_sender
                        .send(StorePageInput::SourcesLoaded(sources))
                        .ok();
                });
            }

            StorePageInput::SourcesLoaded(sources) => {
                self.remote_sources = sources;

                let source_strs: Vec<&str> =
                    self.remote_sources.iter().map(|s| s.as_str()).collect();
                let string_list = gtk::StringList::new(&source_strs);
                self.sources_dropdown.set_model(Some(&string_list));

                let default_source = self
                    .remote_sources
                    .iter()
                    .find(|s| s.as_str() == "Bible.org")
                    .cloned()
                    .or_else(|| self.remote_sources.first().cloned());

                if let Some(source) = default_source {
                    if let Some(pos) = self.remote_sources.iter().position(|s| s == &source) {
                        self.sources_dropdown.set_selected(pos as u32);
                    }
                    self.selected_source = Some(source);
                    sender.input(StorePageInput::RefreshStore);
                }
            }

            StorePageInput::SelectSource(source) => {
                if self.selected_source.as_deref() == Some(&source) {
                    return;
                }
                self.selected_source = Some(source);
                self.search_query.clear();
                self.selected_category = None;
                sender.input(StorePageInput::RefreshStore);
            }

            StorePageInput::RefreshStore => {
                let source = match self.selected_source.clone() {
                    Some(s) => s,
                    None => return,
                };

                self.is_loading = true;
                self.error = None;
                let engine = self.engine.clone();
                let input_sender = sender.input_sender().clone();

                thread::spawn(move || {
                    let modules = engine.fetch_remote_modules(&source);
                    let _installed = engine.refresh_installed_modules();

                    input_sender
                        .send(StorePageInput::ModulesLoaded(Ok(modules)))
                        .ok();
                });
            }

            StorePageInput::ModulesLoaded(result) => {
                self.is_loading = false;
                match result {
                    Ok(modules) => {
                        self.error = None;
                        self.remote_modules = modules;
                        self.filter_and_organize_modules(&sender);
                    }
                    Err(err_msg) => {
                        self.error = Some(err_msg);
                    }
                }
            }

            StorePageInput::SearchChanged(query) => {
                self.search_query = query;
                self.filter_and_organize_modules(&sender);
            }

            StorePageInput::SelectCategory(category) => {
                self.selected_category = Some(category);
                self.sync_category_selection();
                self.rebuild_sections(&sender);
            }

            StorePageInput::InstallModule(module) => {
                let module_name = module.name.clone();
                let source = module.source.clone();

                self.installation_states
                    .insert(module_name.clone(), InstallationStatus::Pending);
                self.notify_sections_status_changed(&module_name, InstallationStatus::Pending);

                let engine = self.engine.clone();
                let input_sender = sender.input_sender().clone();

                glib::spawn_future_local(async move {
                    let task_id = engine.install_module_async(source, module_name.clone());
                    if task_id.is_empty() {
                        input_sender
                            .send(StorePageInput::UpdateInstallationStatus {
                                module_name: module_name.clone(),
                                status: InstallationStatus::Failed(
                                    "Task failed to start".to_string(),
                                ),
                            })
                            .ok();
                    } else {
                        input_sender
                            .send(StorePageInput::TaskStarted {
                                module_name: module_name.clone(),
                                task_id,
                            })
                            .ok();
                    }
                });
            }

            StorePageInput::TaskStarted {
                module_name,
                task_id,
            } => {
                self.active_tasks
                    .insert(module_name.clone(), task_id.clone());

                let engine = self.engine.clone();
                let input_sender = sender.input_sender().clone();

                glib::spawn_future_local(async move {
                    loop {
                        let status = engine.get_task_status(task_id.clone());
                        match status {
                            Some(st) => match st.state {
                                TaskState::Running | TaskState::Queued => {
                                    input_sender
                                        .send(StorePageInput::UpdateInstallationStatus {
                                            module_name: module_name.clone(),
                                            status: InstallationStatus::Installing(st.progress),
                                        })
                                        .ok();
                                }
                                TaskState::Completed => {
                                    let engine_bg = engine.clone();
                                    let sender_bg = input_sender.clone();
                                    let mod_name = module_name.clone();

                                    // Offload heavy SWORD manager directory re-scan to a background thread
                                    std::thread::spawn(move || {
                                        engine_bg.refresh_installed_modules();

                                        sender_bg
                                            .send(StorePageInput::UpdateInstallationStatus {
                                                module_name: mod_name,
                                                status: InstallationStatus::Installed,
                                            })
                                            .ok();
                                    });
                                    break;
                                }
                                TaskState::Failed { error: _ } => {
                                    input_sender
                                        .send(StorePageInput::UpdateInstallationStatus {
                                            module_name: module_name.clone(),
                                            status: InstallationStatus::Failed(
                                                "Installation failed".to_string(),
                                            ),
                                        })
                                        .ok();
                                    break;
                                }
                            },
                            None => {
                                input_sender
                                    .send(StorePageInput::UpdateInstallationStatus {
                                        module_name: module_name.clone(),
                                        status: InstallationStatus::Failed(
                                            "Status not found".to_string(),
                                        ),
                                    })
                                    .ok();
                                break;
                            }
                        }
                        glib::timeout_future(std::time::Duration::from_millis(100)).await;
                    }
                });
            }

            StorePageInput::UpdateInstallationStatus {
                module_name,
                status,
            } => {
                if matches!(
                    status,
                    InstallationStatus::Installed
                        | InstallationStatus::Failed(_)
                        | InstallationStatus::Cancelled
                ) {
                    self.active_tasks.remove(&module_name);
                }
                self.installation_states
                    .insert(module_name.clone(), status.clone());
                self.notify_sections_status_changed(&module_name, status);
            }

            StorePageInput::CancelInstall(module_name) => {
                if let Some(task_id) = self.active_tasks.remove(&module_name) {
                    let engine = self.engine.clone();
                    glib::spawn_future_local(async move {
                        engine.cancel_task(task_id);
                    });
                }
            }

            StorePageInput::HandleSectionOutput(output) => match output {
                ModuleSectionOutput::TileAction(output) => match output {
                    ModuleTileOutput::ActionTriggered(sword_module) => {
                        // Check installation status to determine primary action (Install vs Open)
                        match self.installation_states.get(&sword_module.name) {
                            Some(InstallationStatus::Installed) => {
                                let _ = sender
                                    .output(StorePageOutput::ModuleSelected(sword_module.name));
                            }
                            Some(InstallationStatus::Installing(_))
                            | Some(InstallationStatus::Pending) => {
                                sender.input(StorePageInput::CancelInstall(sword_module.name));
                            }
                            _ => {
                                sender.input(StorePageInput::InstallModule(sword_module));
                            }
                        }
                    }
                    ModuleTileOutput::MenuActionTriggered { module, action } => match action {
                        ModuleMenuAction::OpenInStudy => {
                            let _ = sender.output(StorePageOutput::ModuleSelected(module.name));
                        }
                        ModuleMenuAction::Update => {
                            sender.input(StorePageInput::InstallModule(module));
                        }
                        ModuleMenuAction::Delete => {
                            log::info!("[StorePage] Delete requested for module: {}", module.name);
                            // Trigger uninstall/delete via engine here if required
                        }
                    },
                },
            },
        }
    }
}

impl StorePage {
    fn filter_and_organize_modules(&mut self, sender: &ComponentSender<Self>) {
        let query = self.search_query.to_lowercase();

        let filtered: Vec<SwordModule> = self
            .remote_modules
            .iter()
            .filter(|m| {
                query.is_empty()
                    || m.name.to_lowercase().contains(&query)
                    || m.description.to_lowercase().contains(&query)
                    || m.language.to_lowercase().contains(&query)
            })
            .cloned()
            .collect();

        let mut cats: Vec<String> = filtered.iter().map(|m| m.category.clone()).collect();
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

        self.rebuild_category_chips(sender);
        self.rebuild_sections(sender);
    }

    fn rebuild_category_chips(&mut self, sender: &ComponentSender<Self>) {
        while let Some(child) = self.category_box.first_child() {
            self.category_box.remove(&child);
        }
        self.category_chips.clear();

        let selected = self.selected_category.clone();
        for cat in &self.categories {
            let controller = CategoryChip::builder()
                .launch(CategoryChipInit {
                    name: cat.clone(),
                    is_selected: Some(cat) == selected.as_ref(),
                })
                .forward(sender.input_sender(), |output| match output {
                    CategoryChipOutput::Clicked(name) => StorePageInput::SelectCategory(name),
                });

            self.category_box.append(controller.widget());
            self.category_chips.push(controller);
        }
    }

    fn sync_category_selection(&mut self) {
        let selected = self.selected_category.clone();
        for chip in &self.category_chips {
            let is_sel = Some(&chip.model().name) == selected.as_ref();
            chip.emit(CategoryChipInput::SetSelected(is_sel));
        }
    }

    fn rebuild_sections(&mut self, sender: &ComponentSender<Self>) {
        let query = self.search_query.to_lowercase();

        let filtered: Vec<&SwordModule> = self
            .remote_modules
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
            by_language
                .entry(m.language.clone())
                .or_default()
                .push(m.clone());
        }

        while let Some(child) = self.sections_box.first_child() {
            self.sections_box.remove(&child);
        }
        self.sections.clear();

        // Convert installation states into HashMap for Section init
        let status_map: HashMap<String, InstallationStatus> = self
            .installation_states
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        for (language_name, modules) in by_language {
            let controller = ModuleSection::builder()
                .launch(ModuleSectionInit {
                    language_name,
                    modules,
                    status_map: status_map.clone(),
                    is_library_mode: false,
                })
                .forward(sender.input_sender(), StorePageInput::HandleSectionOutput);

            self.sections_box.append(controller.widget());
            self.sections.push(controller);
        }
    }

    fn notify_sections_status_changed(&self, module_name: &str, status: InstallationStatus) {
        for section in &self.sections {
            section.emit(ModuleSectionInput::ChildInput(
                module_name.to_string(),
                ModuleTileInput::UpdateStatus(status.clone()),
            ));
        }
    }
}
