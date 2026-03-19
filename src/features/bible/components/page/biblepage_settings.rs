use crate::features::core::core::AppSetting::AppSetting;
use gtk::gio::prelude::SettingsExt;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct BiblePageState {
    pub last_module: Option<String>,
    pub last_book: Option<String>,
    pub last_chapter: Option<i32>,
}

pub struct BiblePageSettings;

impl AppSetting for BiblePageSettings {
    fn schema_id() -> &'static str {
        "org.flame.xbible.biblepage"
    }
}

impl BiblePageSettings {
    /// Load the last session state from GSettings
    pub fn load() -> BiblePageState {
        let settings = Self::get_settings();
        let json = settings.string("last-session");
        serde_json::from_str(&json).unwrap_or_default()
    }

    /// Save the current state (module, book, chapter) permanently
    pub fn save(state: BiblePageState) {
        let settings = Self::get_settings();
        if let Ok(json) = serde_json::to_string(&state) {
            let _ = settings.set_string("last-session", &json);
        }
    }

    /// Helper to update just one part of the state without losing the rest
    pub fn update_location(module: Option<String>, book: Option<String>, chapter: Option<i32>) {
        let mut current = Self::load();

        if module.is_some() {
            current.last_module = module;
        }
        if book.is_some() {
            current.last_book = book;
        }
        if chapter.is_some() {
            current.last_chapter = chapter;
        }

        Self::save(current);
    }
}
