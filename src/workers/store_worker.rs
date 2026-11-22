use std::{sync::Arc, time::Duration};

use glib::JoinHandle;
use relm4::prelude::ComponentSender;
use xbible_engine::features::{
    reader::XBibleReader,
    store::{LanguageFacet, Store, StoreItem, StoreShelf, StoreTaskState},
};

use crate::core::engine::SharedEngine;

/// Tracks a single in-flight or completed store-item install task.
///
/// `item_id` is the `StoreItem.id` that was installed — `"kjv"` for a
/// translation, `"theographic:person"` for an entity set, or `"adam-clarke"`
/// for a commentary.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct InstallState {
    pub item_id: String,
    pub kind: String,
    pub fraction: f64,
    pub message: String,
    pub done: bool,
    pub failed: Option<String>,
}

#[derive(Debug)]
pub(crate) struct StoreWorker {
    engine: SharedEngine,
    store: Arc<Store>,
    reader: Arc<XBibleReader>,
    shelves: Vec<StoreShelf>,
    languages: Vec<LanguageFacet>,
    is_loading: bool,
    active_install: Option<InstallState>,
    active_task_id: Option<String>,
    install_handle: Option<JoinHandle<()>>,
}

#[derive(Debug)]
pub(crate) enum StoreWorkerInput {
    /// Mirrors `StoreViewModel.setupEngine()` -> refreshes catalog.
    Setup,
    /// Refreshes remote catalog shelves & language facets.
    RefreshCatalog,
    /// Starts installing a `StoreItem`.
    InstallItem(StoreItem),
    /// Cancels the currently active install task.
    CancelActiveInstall,
    /// Uninstalls any store item by `(id, kind)`.
    UninstallItem { id: String, kind: String },
    /// Translation-only uninstall (guarded against composite `dataset:kind` ids).
    UninstallTranslation(String),
    /// Checks whether a translation id is installed (optimistic cache + reader DB fallback).
    CheckInstalled(String),

    // ── Internal async / polling events ───────────────────────────────────
    #[doc(hidden)]
    CatalogRefreshed(Result<(Vec<StoreShelf>, Vec<LanguageFacet>), String>),
    #[doc(hidden)]
    InstallTaskStarted {
        item_id: String,
        kind: String,
        result: Result<String, String>,
    },
    #[doc(hidden)]
    InstallProgressUpdated(InstallState),
    #[doc(hidden)]
    InstallCancelled,
}

#[derive(Debug, Clone)]
pub(crate) enum StoreWorkerOutput {
    LoadingChanged(bool),
    CatalogUpdated {
        shelves: Vec<StoreShelf>,
        languages: Vec<LanguageFacet>,
    },
    ActiveInstallChanged(Option<InstallState>),
    InstalledStatus {
        translation_id: String,
        installed: bool,
    },
    Error(String),
}

impl relm4::Worker for StoreWorker {
    type Init = SharedEngine;
    type Input = StoreWorkerInput;
    type Output = StoreWorkerOutput;

    fn init(engine: Self::Init, sender: ComponentSender<Self>) -> Self {
        let store = engine.store();
        let reader = engine.reader();

        sender.input(StoreWorkerInput::Setup);

        Self {
            engine,
            store,
            reader,
            shelves: Vec::new(),
            languages: Vec::new(),
            is_loading: false,
            active_install: None,
            active_task_id: None,
            install_handle: None,
        }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            StoreWorkerInput::Setup => {
                debug!("[StoreVM] setupEngine: starting");
                sender.input(StoreWorkerInput::RefreshCatalog);
            }

            StoreWorkerInput::RefreshCatalog => {
                if self.is_loading {
                    return;
                }
                self.is_loading = true;
                let _ = sender.output(StoreWorkerOutput::LoadingChanged(true));
                debug!("[StoreVM] refreshCatalog: starting");

                let store = Arc::clone(&self.store);
                relm4::spawn_local(async move {
                    let result = async {
                        store
                            .refresh()
                            .map_err(|e| format!("Failed to refresh catalog: {e}"))?;

                        let shelves = store.home_feed(None);
                        let languages = store.languages();
                        debug!(
                            "[StoreVM] refreshCatalog: shelves={} languages={}",
                            shelves.len(),
                            languages.len()
                        );
                        Ok((shelves, languages))
                    }
                    .await;

                    sender.input(StoreWorkerInput::CatalogRefreshed(result));
                });
            }

            StoreWorkerInput::CatalogRefreshed(result) => {
                self.is_loading = false;
                let _ = sender.output(StoreWorkerOutput::LoadingChanged(false));

                match result {
                    Ok((shelves, languages)) => {
                        self.shelves = shelves.clone();
                        self.languages = languages.clone();
                        let _ =
                            sender.output(StoreWorkerOutput::CatalogUpdated { shelves, languages });
                    }
                    Err(err) => {
                        error!("[StoreVM] refreshCatalog: FAILED — {err}");
                        let _ = sender.output(StoreWorkerOutput::Error(err));
                    }
                }
            }

            StoreWorkerInput::InstallItem(item) => {
                self.cancel_in_flight_handle();

                let item_id = item.id.clone();
                let kind = item.kind.clone();
                let initial = InstallState {
                    item_id: item_id.clone(),
                    kind: kind.clone(),
                    fraction: 0.0,
                    message: "Starting…".to_string(),
                    done: false,
                    failed: None,
                };
                self.set_active_install(Some(initial), &sender);

                let store = Arc::clone(&self.store);
                let tx = sender.input_sender().clone();
                std::thread::spawn(move || {
                    let result = store.install(item).map_err(|e| e.to_string());
                    let _ = tx.send(StoreWorkerInput::InstallTaskStarted {
                        item_id,
                        kind,
                        result,
                    });
                });
            }

            StoreWorkerInput::InstallTaskStarted {
                item_id,
                kind,
                result,
            } => match result {
                Err(err) => {
                    let mut state = self.active_install.clone().unwrap_or(InstallState {
                        item_id,
                        kind,
                        fraction: 0.0,
                        message: String::new(),
                        done: false,
                        failed: None,
                    });
                    state.failed = Some(err);
                    self.set_active_install(Some(state), &sender);
                }
                Ok(task_id) if task_id.is_empty() => {
                    let mut state = self.active_install.clone().unwrap_or(InstallState {
                        item_id,
                        kind,
                        fraction: 0.0,
                        message: String::new(),
                        done: false,
                        failed: None,
                    });
                    state.failed = Some("Could not start install".to_string());
                    self.set_active_install(Some(state), &sender);
                }
                Ok(task_id) => {
                    self.active_task_id = Some(task_id.clone());

                    let store = Arc::clone(&self.store);
                    let handle = relm4::spawn_local(async move {
                        let mut current = InstallState {
                            item_id,
                            kind,
                            fraction: 0.0,
                            message: "Starting…".to_string(),
                            done: false,
                            failed: None,
                        };

                        loop {
                            match store.poll_task(task_id.clone()) {
                                None => {
                                    glib::timeout_future(Duration::from_millis(500)).await;
                                    continue;
                                }
                                Some(status) => match status.state {
                                    StoreTaskState::Queued => {
                                        current.message = "Queued…".to_string();
                                        sender.input(StoreWorkerInput::InstallProgressUpdated(
                                            current.clone(),
                                        ));
                                    }
                                    StoreTaskState::Running => {
                                        current.fraction = status.progress.fraction;
                                        current.message = status.progress.message;
                                        sender.input(StoreWorkerInput::InstallProgressUpdated(
                                            current.clone(),
                                        ));
                                    }
                                    StoreTaskState::Completed => {
                                        current.done = true;
                                        current.fraction = 1.0;
                                        sender.input(StoreWorkerInput::InstallProgressUpdated(
                                            current,
                                        ));
                                        sender.input(StoreWorkerInput::RefreshCatalog);
                                        return;
                                    }
                                    StoreTaskState::Failed { error } => {
                                        current.failed = Some(error);
                                        sender.input(StoreWorkerInput::InstallProgressUpdated(
                                            current,
                                        ));
                                        return;
                                    }
                                    StoreTaskState::Cancelled => {
                                        sender.input(StoreWorkerInput::InstallCancelled);
                                        return;
                                    }
                                },
                            }

                            glib::timeout_future(Duration::from_millis(500)).await;
                        }
                    });

                    self.install_handle = Some(handle);
                }
            },

            StoreWorkerInput::InstallProgressUpdated(state) => {
                if state.done || state.failed.is_some() {
                    self.active_task_id = None;
                    self.install_handle = None;
                }
                self.set_active_install(Some(state), &sender);
            }

            StoreWorkerInput::InstallCancelled => {
                self.active_task_id = None;
                self.install_handle = None;
                self.set_active_install(None, &sender);
            }

            StoreWorkerInput::CancelActiveInstall => {
                let task_id = self.active_task_id.take();
                self.cancel_in_flight_handle();
                self.set_active_install(None, &sender);

                if let Some(task_id) = task_id {
                    self.store.cancel_task(task_id);
                }
            }

            StoreWorkerInput::UninstallItem { id, kind } => {
                let store = Arc::clone(&self.store);
                relm4::spawn_local(async move {
                    let res: Result<(), String> = match kind.as_str() {
                        "translation" => store
                            .uninstall_translation(id)
                            .map(|_| ())
                            .map_err(|e| e.to_string()),
                        "entity_set" => match split_entity_id(&id) {
                            Ok((_dataset, _entity_kind)) => {
                                Err("entity_set uninstall is not wired in xbible_engine yet"
                                    .to_string())
                            }
                            Err(e) => Err(e),
                        },
                        _ => {
                            if split_entity_id(&id).is_ok() {
                                Err("entity_set uninstall is not wired in xbible_engine yet"
                                    .to_string())
                            } else {
                                store
                                    .uninstall_translation(id)
                                    .map(|_| ())
                                    .map_err(|e| e.to_string())
                            }
                        }
                    };

                    match res {
                        Ok(()) => sender.input(StoreWorkerInput::RefreshCatalog),
                        Err(err) => {
                            let _ = sender.output(StoreWorkerOutput::Error(format!(
                                "Failed to uninstall: {err}"
                            )));
                        }
                    }
                });
            }

            StoreWorkerInput::UninstallTranslation(translation_id) => {
                if translation_id.contains(':') {
                    let _ = sender.output(StoreWorkerOutput::Error(format!(
                        "{translation_id} is a store item id (dataset:kind), not a translation id — call UninstallItem instead."
                    )));
                    return;
                }

                let store = Arc::clone(&self.store);
                relm4::spawn_local(async move {
                    match store.uninstall_translation(translation_id) {
                        Ok(_) => sender.input(StoreWorkerInput::RefreshCatalog),
                        Err(e) => {
                            let _ = sender.output(StoreWorkerOutput::Error(format!(
                                "Failed to uninstall: {e}"
                            )));
                        }
                    }
                });
            }

            StoreWorkerInput::CheckInstalled(translation_id) => {
                let cached = self.shelves.iter().any(|shelf| {
                    shelf
                        .items
                        .iter()
                        .any(|item| item.id == translation_id && item.installed)
                });

                if cached {
                    let _ = sender.output(StoreWorkerOutput::InstalledStatus {
                        translation_id,
                        installed: true,
                    });
                    return;
                }

                let installed = self
                    .reader
                    .is_installed(translation_id.clone())
                    .unwrap_or(false);

                let _ = sender.output(StoreWorkerOutput::InstalledStatus {
                    translation_id,
                    installed,
                });
            }
        }
    }
}

impl StoreWorker {
    fn set_active_install(&mut self, next: Option<InstallState>, sender: &ComponentSender<Self>) {
        if self.active_install != next {
            self.active_install = next.clone();
            let _ = sender.output(StoreWorkerOutput::ActiveInstallChanged(next));
        }
    }

    fn cancel_in_flight_handle(&mut self) {
        if let Some(handle) = self.install_handle.take() {
            handle.abort();
        }
    }
}

fn split_entity_id(id: &str) -> Result<(&str, &str), String> {
    let Some((dataset, kind)) = id.split_once(':') else {
        return Err(format!(
            "{id} is not a valid store item id. Expected a translation id (\"kjv\"), an entity-set id (\"theographic:person\"), or a commentary id (\"adam-clarke\")."
        ));
    };
    if dataset.is_empty() || kind.is_empty() {
        return Err(format!(
            "{id} is not a valid store item id. Expected a translation id (\"kjv\"), an entity-set id (\"theographic:person\"), or a commentary id (\"adam-clarke\")."
        ));
    }
    Ok((dataset, kind))
}
