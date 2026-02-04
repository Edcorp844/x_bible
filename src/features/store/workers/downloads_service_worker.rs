use crate::features::core::module_engine::sword_engine::SwordEngine;
use crate::features::core::pages::store::store_page::{StorePage, StorePageInput};
use crate::features::store::services::download::DownloadService;
use relm4::{ComponentSender, Worker};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug)]
pub struct DownloadWorker {
    service: Arc<DownloadService>,
    subscribers: Vec<ComponentSender<StorePage>>,
}

#[derive(Debug)]
pub enum WorkerInput {
    Subscribe(ComponentSender<StorePage>),
    FetchRemote(String),
    InstallModule { source: String, name: String },
    PollProgress,
}

impl Worker for DownloadWorker {
    type Init = Arc<SwordEngine>;
    type Input = WorkerInput;
    type Output = ();

    fn init(engine: Self::Init, _sender: ComponentSender<Self>) -> Self {
        Self {
            service: Arc::new(DownloadService::new(engine)),
            subscribers: Vec::new(),
        }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            WorkerInput::Subscribe(comp_sender) => {
                self.subscribers.push(comp_sender);
            }

            WorkerInput::FetchRemote(source) => {
                let service = self.service.clone();
                let subs = self.subscribers.clone();

                std::thread::spawn(move || {
                    let modules = service.get_available_modules(&source);
                    for sub in &subs {
                        // Added & to borrow subs
                        sub.input(StorePageInput::UpdateList(modules.clone()));
                    }
                });
            }

            WorkerInput::InstallModule { source, name } => {
                let service = self.service.clone();
                let subs = self.subscribers.clone();
                let worker_sender = sender.clone();

                relm4::gtk::glib::timeout_add_local(Duration::from_millis(100), move || {
                    worker_sender.input(WorkerInput::PollProgress);
                    relm4::gtk::glib::ControlFlow::Continue
                });

                std::thread::spawn(move || {
                    let result = service.download(&source, &name);

                    // We borrow subs and match on a reference to result
                    for sub in &subs {
                        match &result {
                            // Match on &result to avoid moving the error string
                            Ok(_) => sub.input(StorePageInput::UpdateProgress(1.0)),
                            Err(e) => println!("Download failed: {:?}", e),
                        }
                    }
                });
            }

            WorkerInput::PollProgress => {
                let progress = self.service.progress();
                for sub in &self.subscribers {
                    sub.input(StorePageInput::UpdateProgress(progress));
                }
            }
        }
    }
}
