use serde::{Deserialize, Serialize};

use crate::{
    features::core::{
        module_engine::{sword_engine::SwordEngine, sword_module::SwordModule},
        osis_translation_engine::engine::OsisTransilationEngine,
    },
    sword_sys::*,
};

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
    pub language: String,
}

impl Default for Word {
    fn default() -> Self {
        Self {
            text: String::new(),
            lex: None,

            is_red: false,
            is_italic: false,
            is_bold_text: false,
            is_punctuation: false,

            is_first_in_group: false,
            is_last_in_group: false,

            is_title: false,

            language: String::new(),
            note: None,
        }
    }
}

//Verse(group of words)
#[derive(Debug, Clone)]
pub struct Verse {
    pub osis_id: String,
    pub number: i32,
    pub words: Vec<Word>,
    pub notes: Vec<String>,
    pub is_paragraph_start: bool,
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
    pub verses: Vec<Verse>,
    pub text_direction: TextDirection,
}

impl SwordEngine {
    /// Internal helper: Fetches the CURRENT entry pointed to by the module handle and parses it.
    unsafe fn fetch_and_parse_current_entry(
        &self,
        h_mod: isize,
        module: &SwordModule,
        osis_engine: &OsisTransilationEngine,
    ) -> Vec<Section> {
        let current_key =
            unsafe { self.sword_ptr_to_string(org_crosswire_sword_SWModule_getKeyText(h_mod)) };

        // Using getRawEntry for OSIS parsing
        if let Some(raw_osis) =
            unsafe { self.sword_ptr_to_string(org_crosswire_sword_SWModule_getRawEntry(h_mod)) }
        {
            return osis_engine.parse_osis_to_sections(
                module.language.clone(),
                &raw_osis,
                current_key,
            );
        }
        Vec::new()
    }

    /// Fetches a specific reference. Defaults to the first Bible if module is None.
    pub fn get_single_entry(&self, module: Option<&SwordModule>, reference: &str) -> Vec<Section> {
        use std::ffi::CString;
        let osis_engine = OsisTransilationEngine::new();

        let resolved_module = match module {
            Some(m) => m.clone(),
            None => match self
                .get_modules()
                .into_iter()
                .find(|m| m.category == "Biblical Texts")
            {
                Some(m) => m,
                None => return Vec::new(),
            },
        };

        unsafe {
            let mgr_ptr = self.inner.lock().unwrap().mgr;
            let module_name = CString::new(resolved_module.name.as_str()).unwrap();
            let key_ref = CString::new(reference).unwrap();

            let h_mod = org_crosswire_sword_SWMgr_getModuleByName(mgr_ptr, module_name.as_ptr());
            if h_mod == 0 {
                return Vec::new();
            }

            // Set Key
            org_crosswire_sword_SWModule_setKeyText(h_mod, key_ref.as_ptr());

            self.fetch_and_parse_current_entry(h_mod, &resolved_module, &osis_engine)
        }
    }

    /// Fetches a whole chapter by traversing from a starting reference
    pub fn get_whole_chapter(&self, module: &SwordModule, reference: &str) -> Vec<Section> {
        use std::ffi::CString;
        let mut sections = Vec::new();
        let osis_engine = OsisTransilationEngine::new();

        unsafe {
            let mgr_ptr = self.inner.lock().unwrap().mgr;
            let module_name = CString::new(module.name.as_str()).unwrap();
            let key_ref = CString::new(reference).unwrap();

            self.set_global_options(
                &[
                    "Strong's Numbers",
                    "Morphological Tags",
                    "Footnotes",
                    "Cross-references",
                ],
                "On",
            );

            let h_mod = org_crosswire_sword_SWMgr_getModuleByName(mgr_ptr, module_name.as_ptr());
            if h_mod == 0 {
                return sections;
            }

            // Set start position and determine boundary
            org_crosswire_sword_SWModule_setKeyText(h_mod, key_ref.as_ptr());
            let initial_key = self
                .sword_ptr_to_string(org_crosswire_sword_SWModule_getKeyText(h_mod))
                .unwrap_or_default();

            let chapter_boundary = initial_key
                .split(|c| c == ':' || c == '.')
                .next()
                .unwrap_or(&initial_key)
                .to_string();

            loop {
                let current_key = self
                    .sword_ptr_to_string(org_crosswire_sword_SWModule_getKeyText(h_mod))
                    .unwrap_or_default();

                if !current_key.starts_with(&chapter_boundary) {
                    break;
                }

                let verse_sections =
                    self.fetch_and_parse_current_entry(h_mod, module, &osis_engine);
                sections.extend(verse_sections);

                org_crosswire_sword_SWModule_next(h_mod);
                if org_crosswire_sword_SWModule_popError(h_mod) != 0 {
                    break;
                }
            }
        }
        sections
    }
}
