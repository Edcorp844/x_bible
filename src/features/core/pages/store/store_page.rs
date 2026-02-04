use crate::features::core::module_engine::sword_engine::SwordEngine;
use crate::features::core::module_engine::sword_module::SwordModule;
use adw::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, prelude::*, factory::FactoryVecDeque};
use std::sync::Arc;

use crate::features::store::workers::downloads_service_worker::{DownloadWorker, WorkerInput};

#[derive(Debug)]
pub enum StorePageInput {
    RefreshRemote,
    UpdateList(Vec<SwordModule>),
    TriggerDownload(String),
    UpdateProgress(f64),
}

#[derive(Debug)]
pub enum StorePageOutput {
    ToggleSidebar,
}

#[derive(Debug)]
pub struct StorePage {
    worker: relm4::WorkerController<DownloadWorker>,
    remote_modules: FactoryVecDeque<ModuleRow>,
    is_sidebar_visible: bool,
    is_loading: bool,
}

#[relm4::component(pub)]
impl Component for StorePage {
    type Init = (Arc<SwordEngine>, bool);
    type Input = StorePageInput;
    type Output = StorePageOutput;
    type CommandOutput = ();

    view! {
        adw::NavigationPage {
            set_title: "Module Store",
            #[wrap(Some)]
            set_child = &adw::NavigationView {
                push = &adw::NavigationPage {
                    set_title: "Download Modules",
                    #[wrap(Some)]
                    set_child = &adw::ToolbarView {
                        add_top_bar = &adw::HeaderBar {
                            pack_start = &gtk::ToggleButton {
                                set_icon_name: "sidebar-show-symbolic",
                                #[watch]
                                set_active: model.is_sidebar_visible,
                                connect_clicked[sender] => move |_| {
                                    let _ = sender.output(StorePageOutput::ToggleSidebar);
                                }
                            },
                            pack_end = &gtk::Button {
                                set_icon_name: "view-refresh-symbolic",
                                connect_clicked => StorePageInput::RefreshRemote,
                            }
                        },
                        #[wrap(Some)]
                        set_content = &gtk::ScrolledWindow {
                            set_hscrollbar_policy: gtk::PolicyType::Never,
                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_margin_all: 12,
                                set_spacing: 6,

                                gtk::Spinner {
                                    #[watch]
                                    set_visible: model.is_loading,
                                    #[watch]
                                    set_spinning: model.is_loading,
                                    set_halign: gtk::Align::Center,
                                },

                                #[name = "module_list"]
                                gtk::ListBox {
                                    set_selection_mode: gtk::SelectionMode::None,
                                    add_css_class: "boxed-list",
                                    #[watch]
                                    set_visible: !model.is_loading,
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

        // Worker setup
        let worker = DownloadWorker::builder()
            .detach_worker(engine.clone())
            .detach();

        worker.emit(WorkerInput::Subscribe(sender.clone()));

        // Factory for remote modules
        let remote_modules = FactoryVecDeque::builder()
            .launch(gtk::ListBox::default())
            .forward(sender.input_sender(), |name| {
                StorePageInput::TriggerDownload(name)
            });

        let model = StorePage {
            worker,
            remote_modules,
            is_sidebar_visible,
            is_loading: false,
        };

        let widgets = view_output!();
        widgets.module_list.append(&model.remote_modules.widget().clone());

        // Trigger initial fetch
        sender.input(StorePageInput::RefreshRemote);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            StorePageInput::RefreshRemote => {
                self.is_loading = true;
                self.worker.emit(WorkerInput::FetchRemote("CrossWire".to_string()));
            }
            StorePageInput::UpdateList(modules) => {
                self.is_loading = false;
                let mut guard = self.remote_modules.guard();
                guard.clear();
                for m in modules {
                    guard.push_back(m);
                }
            }
            StorePageInput::TriggerDownload(name) => {
                self.worker.emit(WorkerInput::InstallModule {
                    source: "CrossWire".to_string(),
                    name,
                });
            }
            StorePageInput::UpdateProgress(p) => {
                println!("Download progress: {}%", (p * 100.0) as i32);
            }
        }
    }
}

// --- ModuleRow Factory ---
#[derive(Debug)]
pub struct ModuleRow {
    name: String,
    description: String,
}

#[relm4::factory(pub)]
impl FactoryComponent for ModuleRow {
    type Init = SwordModule;
    type Input = ();
    type Output = String;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        adw::ActionRow {
            #[watch]
            set_title: &self.name,
            #[watch]
            set_subtitle: &self.description,
            add_suffix = &gtk::Button {
                set_label: "Download",
                set_valign: gtk::Align::Center,
                add_css_class: "suggested-action",
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            name: init.name,
            description: init.description,
        }
    }
}
