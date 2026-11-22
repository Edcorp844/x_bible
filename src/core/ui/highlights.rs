use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use gettextrs::gettext;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum HighlightColor {
    #[default]
    Yellow,
    Green,
    Blue,
    Pink,
    Purple,
    Orange,
}

impl HighlightColor {
    pub const ALL: [Self; 6] = [
        Self::Yellow,
        Self::Green,
        Self::Blue,
        Self::Pink,
        Self::Purple,
        Self::Orange,
    ];

    pub fn display_name(&self) -> String {
        match self {
            Self::Yellow => gettext("Yellow"),
            Self::Green => gettext("Green"),
            Self::Blue => gettext("Blue"),
            Self::Pink => gettext("Pink"),
            Self::Purple => gettext("Purple"),
            Self::Orange => gettext("Orange"),
        }
    }

    /// RGBA hex strings tuned for translucent GTK/Pango text backgrounds.
    pub fn rgba_hex(&self) -> &'static str {
        match self {
            Self::Yellow => "#F9E26466",
            Self::Green => "#7CE38B59",
            Self::Blue => "#78C2FF59",
            Self::Pink => "#FF8FB159",
            Self::Purple => "#C79BFF59",
            Self::Orange => "#FFB36B59",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum HighlightStyle {
    #[default]
    Solid,
    Underline,
    DashedUnderline,
}

impl HighlightStyle {
    pub const ALL: [Self; 3] = [Self::Solid, Self::Underline, Self::DashedUnderline];

    pub fn display_name(&self) -> String {
        match self {
            Self::Solid => gettext("Solid"),
            Self::Underline => gettext("Underline"),
            Self::DashedUnderline => gettext("Dashed Underline"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistentHighlight {
    /// Unique key formatted as `"{book_id}:{chapter}:{verse}"`
    pub key: String,
    pub color: HighlightColor,
    pub style: HighlightStyle,
}

impl PersistentHighlight {
    pub fn make_key(book_id: &str, chapter: u32, verse: u32) -> String {
        format!("{book_id}:{chapter}:{verse}")
    }

    pub fn new(
        book_id: &str,
        chapter: u32,
        verse: u32,
        color: HighlightColor,
        style: HighlightStyle,
    ) -> Self {
        Self {
            key: Self::make_key(book_id, chapter, verse),
            color,
            style,
        }
    }
}

/// Disk-backed unique-key highlight store mirroring SwiftData's `@Attribute(.unique) var key`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HighlightStore {
    #[serde(default)]
    entries: HashMap<String, PersistentHighlight>,
}

impl HighlightStore {
    pub fn default_path() -> Result<PathBuf, String> {
        let mut dir = glib::user_data_dir();
        dir.push("xbible");
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create data dir {}: {e}", dir.display()))?;
        dir.push("highlights.json");
        Ok(dir)
    }

    pub fn load() -> Result<Self, String> {
        let path = Self::default_path()?;
        Self::load_from(&path)
    }

    pub fn load_from(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let raw = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read highlights {}: {e}", path.display()))?;
        serde_json::from_str(&raw)
            .map_err(|e| format!("Failed to parse highlights {}: {e}", path.display()))
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::default_path()?;
        self.save_to(&path)
    }

    pub fn save_to(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create dir {}: {e}", parent.display()))?;
        }
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize highlights: {e}"))?;
        fs::write(path, json)
            .map_err(|e| format!("Failed to write highlights {}: {e}", path.display()))
    }

    pub fn get(&self, book_id: &str, chapter: u32, verse: u32) -> Option<&PersistentHighlight> {
        let key = PersistentHighlight::make_key(book_id, chapter, verse);
        self.entries.get(&key)
    }

    pub fn upsert(&mut self, highlight: PersistentHighlight) -> Result<(), String> {
        self.entries.insert(highlight.key.clone(), highlight);
        self.save()
    }

    pub fn remove(&mut self, book_id: &str, chapter: u32, verse: u32) -> Result<bool, String> {
        let key = PersistentHighlight::make_key(book_id, chapter, verse);
        let existed = self.entries.remove(&key).is_some();
        if existed {
            self.save()?;
        }
        Ok(existed)
    }
}
