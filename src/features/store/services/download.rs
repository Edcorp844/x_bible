use std::sync::Arc;
use crate::features::core::module_engine::sword_engine::SwordEngine;
use crate::features::core::module_engine::sword_module::SwordModule;

#[derive(Debug)]
pub struct DownloadService {
    engine: Arc<SwordEngine>,
}

impl DownloadService {
    pub fn new(engine: Arc<SwordEngine>) -> Self {
        Self { engine }
    }

    /// 1. Get the list of remote repositories (e.g., ["CrossWire", "IBT"])
   // pub fn get_remote_sources(&self) -> Vec<String> {
        //self.engine.refresh_remote_sources();
       // self.engine.get_remote_source_list()
   // }

    /// 2. Get metadata for modules available at a specific remote source
    pub fn get_available_modules(&self, source_name: &str) -> Vec<SwordModule> {
        self.engine.fetch_remote_modules(source_name)
    }

    /// 3. Check progress of an active download
    pub fn progress(&self) -> f64 {
        self.engine.get_download_progress()
    }

    /// 4. Execute the download
    pub fn download(&self, source: &str, module_name: &str) -> Result<(), String> {
        let res = self.engine.install_remote_module(source, module_name);
        if res == 0 { 
            Ok(()) 
        } else { 
            Err(format!("Download failed with SWORD error code: {}", res)) 
        }
    }
}