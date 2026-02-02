use serde::{Deserialize, Serialize};

/// How a segment should be rendered or interpreted
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SegmentStyle {
    Plain,
    Added,     // Supplied words (italics / brackets)
    RedLetter, // Words of Christ
    Note,      // Footnotes or annotations
}

/// Lexical metadata attached to a word
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LexicalInfo {
    pub strongs: Vec<String>,   // "G3056"
    pub lemma: Option<String>,  // "λόγος"
    pub gloss: Option<String>,  // "word, speech"
    pub morph: Option<String>,
}


/// A single renderable word or punctuation mark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Word {
    pub text: String,
    pub style: SegmentStyle,
    pub is_red: bool,
    pub is_italic: bool,
    pub is_bold_text: bool,

    /// Lexicon & dictionary hooks
    pub lex: Option<LexicalInfo>,

    /// Grouping flags (for Added / RedLetter spans)
    pub is_first_in_group: bool,
    pub is_last_in_group: bool,

    /// Layout hint
    pub is_punctuation: bool,
}

/// A full verse, UI-agnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Verse {
    pub osis_id: String,
    pub number: i32,

    pub words: Vec<Word>,
    pub notes: Vec<String>,

    /// Paragraph indentation hint
    pub is_paragraph_start: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub osis_id: String,
    pub title: String,
    pub chapters: Vec<Chapter>,
    pub canonical: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub title: String,
    pub osis_ref: String,
    pub number: String,
    pub verses: Vec<Verse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BibleVersion {
    pub osis_id: String,    // e.g., "KJV"
    pub title: String,      // e.g., "King James Version (1769)"
    pub identifier: String, // e.g., "Bible.KJV"
    pub scope: String,      // e.g., "Gen-Rev"
    pub ref_system: String, // e.g., "Bible.KJV"
}

#[derive(Debug)]
pub enum HtmlEvent {
    Text(String),
    Strong(String),     // "G3056"
    Morph(String),      // "V-PAI-3S"
    Note(String),
    RedStart,
    RedEnd,
    AddedWord,
}