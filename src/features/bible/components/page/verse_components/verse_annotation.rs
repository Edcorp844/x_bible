use gtk::gio::prelude::SettingsExt;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::features::core::core::app_setting::AppSetting;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct VerseAnnotation {
    pub color: Option<String>,
    pub note: Option<String>,
    pub is_bookmarked: bool,
}

impl VerseAnnotation {
    pub fn new() -> Self {
        Self {
            color: None,
            note: None,
            is_bookmarked: false,
        }
    }
}

pub struct AnnotationSettings;

impl AppSetting for AnnotationSettings {
    fn schema_id() -> &'static str {
        "org.flame.xbible.verseannotaion"
    }
}

impl AnnotationSettings {
    /// The "Fast" way: Load everything into memory at once
    pub fn load_all() -> HashMap<String, VerseAnnotation> {
        let settings = Self::get_settings();
        let json = settings.string("verse-annotations");
        serde_json::from_str(&json).unwrap_or_default()
    }

    /// Save or Update a single verse without losing other data
    pub fn save_verse(verse_id: &str, annotation: VerseAnnotation) {
        let settings = Self::get_settings();
        let mut all = Self::load_all();

        all.insert(verse_id.to_string(), annotation);

        if let Ok(json) = serde_json::to_string(&all) {
            let _ = settings.set_string("verse-annotations", &json);
        }
    }
}

pub type Annotations = HashMap<String, VerseAnnotation>;
