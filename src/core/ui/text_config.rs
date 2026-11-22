use std::{
    fs,
    path::{Path, PathBuf},
};

use gettextrs::gettext;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum AddedWordsStyle {
    #[default]
    Brackets,
    Italics,
    Underline,
    None,
}

impl AddedWordsStyle {
    pub const ALL: [Self; 4] = [Self::Brackets, Self::Italics, Self::Underline, Self::None];

    pub fn display_name(&self) -> String {
        match self {
            Self::Brackets => gettext("Brackets"),
            Self::Italics => gettext("Italics"),
            Self::Underline => gettext("Underline"),
            Self::None => gettext("None"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum SystemDesign {
    #[default]
    Default,
    Serif,
    Rounded,
    Monospaced,
}

impl SystemDesign {
    pub const ALL: [Self; 4] = [Self::Default, Self::Serif, Self::Rounded, Self::Monospaced];

    pub fn display_name(&self) -> String {
        match self {
            Self::Default => gettext("Default"),
            Self::Serif => gettext("Serif"),
            Self::Rounded => gettext("Rounded"),
            Self::Monospaced => gettext("Monospaced"),
        }
    }

    /// Maps the design choice to a CSS/Pango generic font family fallback for GTK4 rendering.
    pub fn css_font_family(&self) -> &'static str {
        match self {
            Self::Default => "sans-serif",
            Self::Serif => "serif",
            Self::Rounded => "ui-rounded, sans-serif",
            Self::Monospaced => "monospace",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum VerseLayout {
    #[default]
    Paragraph,
    VersePerLine,
}

impl VerseLayout {
    pub const ALL: [Self; 2] = [Self::Paragraph, Self::VersePerLine];

    pub fn display_name(&self) -> String {
        match self {
            Self::Paragraph => gettext("Paragraph"),
            Self::VersePerLine => gettext("Verse per Line"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct TextConfig {
    pub font_size_percent: u32,
    pub use_system_font: bool,
    pub system_design: SystemDesign,
    pub font_family_name: String,
    pub font_postscript_name: String,
    pub line_spacing: f64,
    pub word_spacing: f64,
    pub verse_spacing: f64,
    pub justify: bool,
    pub show_red_words: bool,
    pub show_strongs: bool,
    pub show_lemma: bool,
    pub show_morph: bool,
    pub show_verse_numbers: bool,
    pub show_verse_notes: bool,
    pub show_word_notes: bool,
    pub added_words_style: AddedWordsStyle,
    pub verse_layout: VerseLayout,
}

impl Default for TextConfig {
    fn default() -> Self {
        Self {
            font_size_percent: 100,
            use_system_font: true,
            system_design: SystemDesign::Default,
            font_family_name: String::new(),
            font_postscript_name: String::new(),
            line_spacing: 1.5,
            word_spacing: 8.0,
            verse_spacing: 1.5,
            justify: false,
            show_red_words: true,
            show_strongs: true,
            show_lemma: true,
            show_morph: true,
            show_verse_numbers: true,
            show_verse_notes: true,
            show_word_notes: true,
            added_words_style: AddedWordsStyle::Brackets,
            verse_layout: VerseLayout::Paragraph,
        }
    }
}

impl TextConfig {
    pub fn default_path() -> Result<PathBuf, String> {
        let mut dir = glib::user_config_dir();
        dir.push("xbible");
        fs::create_dir_all(&dir)
            .map_err(|e| format!("Failed to create config dir {}: {e}", dir.display()))?;
        dir.push("text_config.json");
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
            .map_err(|e| format!("Failed to read text config {}: {e}", path.display()))?;
        serde_json::from_str(&raw)
            .map_err(|e| format!("Failed to parse text config {}: {e}", path.display()))
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
            .map_err(|e| format!("Failed to serialize text config: {e}"))?;
        fs::write(path, json)
            .map_err(|e| format!("Failed to write text config {}: {e}", path.display()))
    }

    /// Computes effective point size from a base body size (e.g. 14.0 pt).
    pub fn effective_font_pt(&self, base_pt: f64) -> f64 {
        base_pt * (self.font_size_percent.clamp(50, 300) as f64 / 100.0)
    }

    /// Resolves the active font family name for Pango / GTK CSS.
    pub fn resolved_font_family(&self) -> &str {
        if !self.use_system_font && !self.font_family_name.trim().is_empty() {
            self.font_family_name.trim()
        } else {
            self.system_design.css_font_family()
        }
    }
}
