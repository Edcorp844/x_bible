use crate::{features::core::module_engine::sword_engine::SwordEngine, sword_sys::*};
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

        // Use your existing list of module names to avoid re-scanning the whole manager
        let dict_modules = self.get_dictionary_modules();

        for module in dict_modules {
            let mut search_keys = query.strongs.clone();
            if !query.word.is_empty() {
                search_keys.push(query.word.clone());
            }

            for key in search_keys {
                // TRY PADDING: If it's a Strong's number, try H06440 instead of H6440
                let keys_to_try = if key.starts_with('H') || key.starts_with('G') {
                    if key.len() < 6 {
                        vec![key.clone(), format!("{}{:0>4}", &key[0..1], &key[1..])]
                    } else {
                        vec![key.clone()]
                    }
                } else {
                    vec![key.clone(), key.to_uppercase(), key.to_lowercase()]
                };

                for k in keys_to_try {
                    if let Some(definition) = self.get_dictionary_entry_direct(&module.name, &k) {
                        results.push(DictionaryResult {
                            module_name: module.name.clone(),
                            key: k.clone(),
                            definition,
                        });
                        break; // Stop after first successful key match in this module
                    }
                }
            }
        }
        DictionaryResponse { results }
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

            // 1. Force a Search/Set
            org_crosswire_sword_SWModule_setKeyText(h_mod, c_key.as_ptr());

            // 2. Check if the key actually exists in this module
            if org_crosswire_sword_SWModule_popError(h_mod) != 0 {
                return None;
            }

            // 3. Render the text (Essential for Webster and Lexicons)
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

        // 1. First, handle the XML/HTML escaping to prevent parsing crashes
        // We temporarily replace & to avoid double-escaping later
        let mut formatted = raw_text.replace("&amp;", "&").replace("&", "&amp;");

        // 2. STRIP DISALLOWED ATTRIBUTES
        // Dictionaries love <span class="def"> or <div align="center">.
        // We convert these to plain <span> tags or remove them.
        formatted = formatted
            .replace("<span class=\"def\">", "<span>")
            .replace("<span class='def'>", "<span>")
            .replace("<div", "<span") // Pango doesn't like <div>
            .replace("</div>", "</span>");

        // 3. CONVERT COMMON DICTIONARY TAGS
        formatted = formatted
            // Line breaks
            .replace("<br/>", "\n")
            .replace("<br />", "\n")
            .replace("<br>", "\n")
            .replace("<p>", "\n\n")
            .replace("</p>", "")
            // Formatting (Standardize to Pango-supported attributes)
            .replace("<i>", "<span style='italic'>")
            .replace("</i>", "</span>")
            .replace("<b>", "<span weight='bold'>")
            .replace("</b>", "</span>")
            // Handle Strong's/Lexicon specific tags often found in OSIS
            .replace("<entry>", "")
            .replace("</entry>", "\n")
            .replace("<def>", "")
            .replace("</def>", "")
            .replace("<lg>", "")
            .replace("</lg>", "")
            .replace("<l>", "")
            .replace("</l>", "\n");

        // 4. CLEANUP: Some dictionaries use non-standard characters
        // and sometimes leave empty spans
        formatted = formatted.replace("<span></span>", "").replace("  ", " ");

        formatted.trim().to_string()
    }
}
