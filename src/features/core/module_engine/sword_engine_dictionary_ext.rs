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
    pub strongs: Vec<String>, // Now a Vec
    pub language: String,
}

impl SwordEngine {
    pub fn lookup_dictionary(&self, query: DictionaryQuery) -> DictionaryResponse {
        let mut results = Vec::new();

        // 1. Get dictionary modules filtered by language
        let modules = self.get_modules();
        let target_modules: Vec<_> = modules
            .iter()
            .filter(|m| {
                let is_dict = m.category == "Dictionary" || m.category == "Dictionaries";
                // Match language if query.language is not empty
                let lang_match = query.language.is_empty() || m.language == query.language;
                is_dict && lang_match
            })
            .collect();

        for module in target_modules {
            // 2. Prepare the list of keys to search (Strongs first, then the word)
            let mut search_keys = query.strongs.clone();
            if !query.word.is_empty() {
                search_keys.push(query.word.clone());
            }

            for key in search_keys {
                if let Some(definition) = self.get_dictionary_entry(&module.name, &key) {
                    results.push(DictionaryResult {
                        module_name: module.name.to_string(),
                        key: key.clone(),
                        definition,
                    });
                    // Optional: break if you only want the first match per module
                    // break;
                }
            }
        }

        DictionaryResponse { results }
    }

    fn get_dictionary_entry(&self, module_name: &str, key: &str) -> Option<String> {
        // Use a single lock for the duration of the lookup
        let inner = self.inner.lock().unwrap();
        let c_mod = CString::new(module_name).ok()?;
        let c_key = CString::new(key).ok()?;

        unsafe {
            let module_ptr = org_crosswire_sword_SWMgr_getModuleByName(inner.mgr, c_mod.as_ptr());
            if module_ptr == 0 {
                return None;
            }

            // Set the key in the module cursor
            org_crosswire_sword_SWModule_setKeyText(module_ptr, c_key.as_ptr());

            // Check if SWORD actually found the key
            if org_crosswire_sword_SWModule_popError(module_ptr) != 0 {
                return None;
            }

            // Use getRawEntry for the definition
            let raw_ptr = org_crosswire_sword_SWModule_getRawEntry(module_ptr);
            if raw_ptr.is_null() {
                return None;
            }

            let text = CStr::from_ptr(raw_ptr).to_string_lossy().into_owned();

            if text.trim().is_empty() {
                None
            } else {
                Some(text)
            }
        }
    }
}
