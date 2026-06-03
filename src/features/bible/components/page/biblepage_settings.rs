use crate::features::core::core::app_setting::AppSetting;
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
}
