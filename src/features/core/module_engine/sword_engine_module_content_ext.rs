use serde::{Deserialize, Serialize};
use std::ffi::CString;

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
    pub strongs: Vec<String>,
    pub lemma: Option<String>,
    pub gloss: Option<String>,
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

// Verse (group of words)
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
pub struct Subsection {
    pub title: Vec<Word>,   // The "Subsection" heading (H4 level)
    pub verses: Vec<Verse>,
}

#[derive(Debug, Clone)]
pub struct Section {
    pub title: Vec<Word>,      // The "Section" heading (H3 level)
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
        let current_key = unsafe { 
            self.sword_ptr_to_string(org_crosswire_sword_SWModule_getKeyText(h_mod)) 
        };

        if let Some(raw_osis) = unsafe { 
            self.sword_ptr_to_string(org_crosswire_sword_SWModule_getRawEntry(h_mod)) 
        } {
            return osis_engine.parse_osis_to_sections(
                module.language.clone(),
                &raw_osis,
                current_key,
            );
        }
        Vec::new()
    }

    /// Helper to fetch introductions/titles stored at Verse 0
    fn fetch_intro(&self, h_mod: isize, key_str: &str) -> Option<(String, String)> {
        unsafe {
            let c_key = CString::new(key_str).unwrap();
            org_crosswire_sword_SWModule_setKeyText(h_mod, c_key.as_ptr());
            
            let raw = self.sword_ptr_to_string(org_crosswire_sword_SWModule_getRawEntry(h_mod))?;
            if raw.trim().is_empty() {
                return None;
            }
            let actual_key = self.sword_ptr_to_string(org_crosswire_sword_SWModule_getKeyText(h_mod))?;
            Some((actual_key, raw))
        }
    }

    /// Fetches a specific reference. Defaults to the first Bible if module is None.
    pub fn get_single_entry(&self, module: Option<&SwordModule>, reference: &str) -> Vec<Section> {
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

            org_crosswire_sword_SWModule_setKeyText(h_mod, key_ref.as_ptr());
            self.fetch_and_parse_current_entry(h_mod, &resolved_module, &osis_engine)
        }
    }

    /// Fetches a whole chapter by traversing from a starting reference
    pub fn get_whole_chapter(&self, module: &SwordModule, reference: &str) -> Vec<Section> {
        let mut raw_entries = Vec::new();

        unsafe {
            let inner = self.inner.lock().unwrap();
            let h_mod = org_crosswire_sword_SWMgr_getModuleByName(
                inner.mgr,
                CString::new(module.name.as_str()).unwrap().as_ptr(),
            );

            if h_mod == 0 {
                return Vec::new();
            }

            // 1. Position at start to identify where we are
            let c_ref = CString::new(reference).unwrap();
            org_crosswire_sword_SWModule_setKeyText(h_mod, c_ref.as_ptr());

            let initial_key = self
                .sword_ptr_to_string(org_crosswire_sword_SWModule_getKeyText(h_mod))
                .unwrap_or_default();

            let (target_book, target_chapter) = match self.parse_reference(&initial_key) {
                Some(val) => val,
                None => return Vec::new(),
            };

            // 2. Try fetching intros (Book intro if Ch 1, otherwise Chapter intro)
            if target_chapter == "1" {
                if let Some(intro) = self.fetch_intro(h_mod, &format!("{} 0:0", target_book)) {
                    raw_entries.push(intro);
                }
            }
            if let Some(intro) = self.fetch_intro(h_mod, &format!("{} {}:0", target_book, target_chapter)) {
                raw_entries.push(intro);
            }

            // 3. Collect standard verses
            let start_ref = CString::new(format!("{} {}:1", target_book, target_chapter)).unwrap();
            org_crosswire_sword_SWModule_setKeyText(h_mod, start_ref.as_ptr());

            loop {
                let current_key = self
                    .sword_ptr_to_string(org_crosswire_sword_SWModule_getKeyText(h_mod))
                    .unwrap_or_default();

                let (curr_book, curr_chap) = match self.parse_reference(&current_key) {
                    Some(val) => val,
                    None => break,
                };

                if curr_book != target_book || curr_chap != target_chapter {
                    break;
                }

                if let Some(raw_osis) = self.sword_ptr_to_string(org_crosswire_sword_SWModule_getRawEntry(h_mod)) {
                println!("[=] {:?}", raw_osis);
                    raw_entries.push((current_key, raw_osis));
                }

                org_crosswire_sword_SWModule_next(h_mod);
                if org_crosswire_sword_SWModule_popError(h_mod) != 0 {
                    break;
                }
            }
        }

        if raw_entries.is_empty() {
            return Vec::new();
        }

        let engine = OsisTransilationEngine::new();
        engine.parse_osis_list_to_sections(module.language.clone(), raw_entries)
    }

    /// Helper to reliably split "1 John 3:1" into ("1 john", "3")
    fn parse_reference(&self, full_key: &str) -> Option<(String, String)> {
        let last_space_idx = full_key.rfind(' ')?;
        let book = full_key[..last_space_idx].to_lowercase();
        let rest = &full_key[last_space_idx + 1..];

        let chapter = match rest.find(':') {
            Some(colon_idx) => &rest[..colon_idx],
            None => rest,
        };

        Some((book, chapter.to_string()))
    }
}
