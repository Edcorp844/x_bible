use crate::{features::core::module_engine::sword_engine::SwordEngine, sword_sys::*};
use html2pango::markup_html;
use regex::Regex;
use std::ffi::{CStr, CString};

#[derive(Debug, Clone)]
pub struct DictionaryResult {
    pub module_name: String,
    pub key: String,
    pub definition: String,
}

#[derive(Debug, Clone)]
pub struct DictionaryResponse {
    pub results: Vec<DictionaryResult>,
}

#[derive(Debug, Clone)]
pub struct DictionaryQuery {
    pub word: String,
    pub strongs: Vec<String>,
    pub language: String,
}

impl SwordEngine {
    pub fn lookup_dictionary(&self, query: DictionaryQuery) -> DictionaryResponse {
        let mut results = Vec::new();
        let dict_modules = self.get_dictionary_modules();

        let language_modules: Vec<_> = dict_modules
            .into_iter()
            .filter(|module| module.language.to_lowercase() == query.language.to_lowercase())
            .collect();

        for module in language_modules {
            let mut search_keys = Vec::new();
            if !query.word.is_empty() {
                search_keys.push(query.word.clone());
            }

            for key in search_keys {
                // We try the key variations
                let keys_to_try = vec![key.clone(), key.to_uppercase(), key.to_lowercase()];

                for k in keys_to_try {
                    // 1. Attempt to get the entry
                    if let Some((actual_key, definition)) =
                        self.get_dictionary_entry_with_key_check(&module.name, &k)
                    {
                        // 2. STRICT CHECK: Does the actual key from SWORD match our requested key?
                        // We use case-insensitive comparison to be safe, or exact if you prefer.
                        if actual_key.to_lowercase() == k.to_lowercase() {
                            results.push(DictionaryResult {
                                module_name: module.description.clone(),
                                key: actual_key, // Use the official key from the module
                                definition: self.format_for_pango(&definition),
                            });
                            break; // Found the exact match for this module
                        }
                    }
                }
            }
        }
        DictionaryResponse { results }
    }

    fn get_dictionary_entry_with_key_check(
        &self,
        module_name: &str,
        key: &str,
    ) -> Option<(String, String)> {
        let inner = self.inner.lock().unwrap();
        unsafe {
            let c_mod = CString::new(module_name).ok()?;
            let c_key = CString::new(key).ok()?;
            let h_mgr = inner.mgr;
            let h_mod = org_crosswire_sword_SWMgr_getModuleByName(h_mgr, c_mod.as_ptr());

            if h_mod == 0 {
                return None;
            }

            // Set the key
            org_crosswire_sword_SWModule_setKeyText(h_mod, c_key.as_ptr());

            // Check for SWORD errors (Key not found)
            if org_crosswire_sword_SWModule_popError(h_mod) != 0 {
                return None;
            }

            // 3. GET ACTUAL KEY: See where SWORD actually landed
            let actual_key_ptr = org_crosswire_sword_SWModule_getKeyText(h_mod);
            if actual_key_ptr.is_null() {
                return None;
            }
            let actual_key = CStr::from_ptr(actual_key_ptr)
                .to_string_lossy()
                .into_owned();

            // 4. GET RENDERED TEXT
            let text_ptr = org_crosswire_sword_SWModule_renderText(h_mod);
            if text_ptr.is_null() {
                return None;
            }
            let text = CStr::from_ptr(text_ptr).to_string_lossy().into_owned();

            if text.trim().is_empty() {
                None
            } else {
                Some((actual_key, text))
            }
        }
    }

    fn get_dictionary_entry_direct(&self, module_name: &str, key: &str) -> Option<String> {
        let inner = self.inner.lock().unwrap();
        unsafe {
            let c_mod = CString::new(module_name).ok()?;
            let c_key = CString::new(key).ok()?;
            let h_mgr = inner.mgr;
            let h_mod = org_crosswire_sword_SWMgr_getModuleByName(h_mgr, c_mod.as_ptr());

            if h_mod == 0 {
                return None;
            }

            org_crosswire_sword_SWModule_setKeyText(h_mod, c_key.as_ptr());
            if org_crosswire_sword_SWModule_popError(h_mod) != 0 {
                return None;
            }

            let text_ptr = org_crosswire_sword_SWModule_renderText(h_mod);
            if text_ptr.is_null() {
                return None;
            }

            let text = CStr::from_ptr(text_ptr).to_string_lossy().into_owned();
            if text.trim().is_empty() {
                None
            } else {
                Some(text)
            }
        }
    }

    pub fn format_for_pango(&self, raw_text: &str) -> String {
        if raw_text.is_empty() {
            return String::new();
        }

        // 1. Map SWORD breaks to a "Safe" placeholder that html2pango won't touch
        let processed_html = raw_text
            .replace("<!P>", " [[BLOCK_BREAK]] ")
            .replace("<!/P>", "")
            .replace("<br/>", " [[LINE_BREAK]] ")
            .replace("<br />", " [[LINE_BREAK]] ")
            .replace("<orth", "<span class='orth'")
            .replace("</orth>", "</span> [[BLOCK_BREAK]] ")
            .replace("<pos", "<span class='pos'")
            .replace("</pos>", "</span>")
            .replace("<entryFree", "<span class='entryFree'")
            .replace("</entryFree>", "</span>");

        // 2. Convert to Pango
        let mut pango_markup = match markup_html(&processed_html) {
            Ok(m) => m,
            Err(_) => glib::markup_escape_text(raw_text).to_string(),
        };

        // 3. THE CRITICAL STEP: Inject hard Unicode separators
        // \u{2028} is the "Line Separator" - Pango MUST break here.
        // \u{2029} is the "Paragraph Separator" - adds even more space.
        pango_markup = pango_markup
            .replace("[[BLOCK_BREAK]]", "\u{2029}")
            .replace("[[LINE_BREAK]]", "\u{2028}");

        // 4. Final Styling
        let styled = pango_markup
            .replace(
                "class='entryFree'",
                "size='large' weight='bold' foreground='#2e3436'",
            )
            .replace("class='pos'", "style='italic' foreground='#3465a4'")
            .replace(
                "class='orth'",
                "size='x-large' weight='heavy' foreground='#000000'",
            )
            .replace("<sup>", "<span rise='6000' size='x-small' weight='bold'>")
            .replace("</sup>", "</span>");

        self.process_scripts_in_pango(&styled)
    }

    fn process_scripts_in_pango(&self, text: &str) -> String {
        let mut result = String::new();
        let parts = text.split_inclusive(['<', '>']);

        for part in parts {
            if part.starts_with('<') && part.ends_with('>') {
                result.push_str(part);
            } else {
                for word in part.split_inclusive(char::is_whitespace) {
                    let has_hebrew = word.chars().any(|c| ('\u{0590}'..='\u{05FF}').contains(&c));
                    let has_greek = word.chars().any(|c| ('\u{0370}'..='\u{03FF}').contains(&c));

                    if has_hebrew {
                        result.push_str(&format!(
                            "<span font='SBL Hebrew, David, Serif' size='large'>{}</span>",
                            word
                        ));
                    } else if has_greek {
                        result.push_str(&format!(
                            "<span font='SBL Greek, Gentium, Serif' size='large'>{}</span>",
                            word
                        ));
                    } else {
                        result.push_str(word);
                    }
                }
            }
        }
        result
    }
}
