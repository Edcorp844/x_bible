use std::fmt;

use gtk::gio::Settings;
use serde::{Deserialize, Serialize};

/// How a segment should be rendered or interpreted
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SegmentStyle {
    Plain,
    Added,     // Supplied words (italics / brackets)
    RedLetter, // Words of Christ
    Note,      // Footnotes or annotations
}

/// Lexical metadata attached to a word
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LexicalInfo {
    pub strongs: Vec<String>,  // "G3056"
    pub lemma: Option<String>, // "λόγος"
    pub gloss: Option<String>, // "word, speech"
    pub morph: Vec<String>,
}

/// A single renderable word or punctuation mark
#[derive(Debug, Clone)]
pub struct Word {
    pub text: String,

    pub style: SegmentStyle,
    pub is_red: bool,
    pub is_italic: bool,
    pub is_bold_text: bool,

    /// Lexicon & dictionary hooks
    pub lex: Option<LexicalInfo>,
    pub note: Option<String>,

    /// Grouping flags (for Added / RedLetter spans)
    pub is_first_in_group: bool,
    pub is_last_in_group: bool,

    /// Layout hint
    pub is_punctuation: bool,

    pub is_title: bool,
}

/// A full verse, UI-agnostic
#[derive(Debug, Clone)]
pub struct Verse {
    pub osis_id: String,
    pub number: i32,

    pub words: Vec<Word>,
    pub notes: Vec<String>,

    /// Paragraph indentation hint
    pub is_paragraph_start: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TitleStyle {
    H1,
    H2,
    H3,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TextDirection {
    Rtl,
    Ltr,
}

impl TextDirection {
    pub fn to_gtk_text_direction(&self) -> gtk::TextDirection {
        match self {
            Self::Ltr => gtk::TextDirection::Ltr,
            Self::Rtl => gtk::TextDirection::Rtl,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Section {
    pub title: Vec<Word>,
    pub title_style: TitleStyle,
    pub verses: Vec<Verse>,
    pub text_direction: TextDirection,
}

impl Default for Word {
    fn default() -> Self {
        Self {
            text: String::new(),
            lex: None,

            style: SegmentStyle::Plain,
            is_red: false,
            is_italic: false,
            is_bold_text: false,
            is_punctuation: false,

            is_first_in_group: false,
            is_last_in_group: false,

            is_title: false,
            note: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddedWordStyle {
    Italic,
    Brackets,
}

impl AddedWordStyle {
    /// Converts a string from GSettings back into the Enum
    pub fn from_string(s: &str) -> Self {
        match s {
            "Italic" => Self::Italic,
            "Brackets" => Self::Brackets,
            _ => Self::Brackets,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Brackets => format!("Brackets"),
            Self::Italic => format!("Italic"),
        }
    }

    pub fn all() -> Vec<Self> {
        vec![Self::Italic, Self::Brackets]
    }
}

// This allows to call .to_string() on the enum
impl fmt::Display for AddedWordStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Italic => "Italic",
            Self::Brackets => "Brackets",
        };
        write!(f, "{s}")
    }
}

#[derive(Clone, Debug)]
pub struct PageDisplayConfig {
    pub settings: Settings,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AvailableFonts {
    Sans,
    Serif,
    Monospace,
    TimesNewRoman,
    System,
}

impl AvailableFonts {
    pub fn all() -> Vec<Self> {
        vec![
            Self::System,
            Self::Sans,
            Self::Serif,
            Self::Monospace,
            Self::TimesNewRoman,
        ]
    }

    pub fn from_string(s: &str) -> Self {
        match s {
            "System" => Self::System,
            "Sans" => Self::Sans,
            "Serif" => Self::Serif,
            "Monospace" => Self::Monospace,
            "Times New Roman" => Self::TimesNewRoman,
            _ => Self::System,
        }
    }
}

impl fmt::Display for AvailableFonts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::System => "System",
            Self::Sans => "Sans",
            Self::Serif => "Serif",
            Self::Monospace => "Monospace",
            Self::TimesNewRoman => "Times New Roman",
        };
        write!(f, "{s}")
    }
}
