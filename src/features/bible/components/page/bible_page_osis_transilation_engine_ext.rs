use crate::{
    features::{
        bible::components::page::{biblepage_model::BiblePage, helpers::Section},
        core::osis_translation_engine::engine::OsisTransilationEngine,
    },
    sword_sys::*,
};

impl BiblePage {
    pub fn load_reference(&mut self, reference: &str) {
        let sections = self.render_content_to_sections(reference);
        let mut guard = self.sections.guard();
        guard.clear();
        for section in sections {
            guard.push_back((section, self.config.clone(), self.annotations.clone()));
        }
    }

    pub fn render_content_to_sections(&self, reference: &str) -> Vec<Section> {
        use std::ffi::CString;
        let mut verses = Vec::new();
        let osis_engine = OsisTransilationEngine::new();

        unsafe {
            let key_ref = CString::new(reference).unwrap();
            let module_name = CString::new(self.module.as_str()).unwrap();

            let options = [
                "Strong's Numbers",
                "Morphological Tags",
                "Footnotes",
                "Cross-references",
            ];
            let on = CString::new("On").unwrap();

            for opt in options {
                let opt_c = CString::new(opt).unwrap();
                org_crosswire_sword_SWMgr_setGlobalOption(
                    self.mgr_ptr,
                    opt_c.as_ptr(),
                    on.as_ptr(),
                );
            }

            let h_mod =
                org_crosswire_sword_SWMgr_getModuleByName(self.mgr_ptr, module_name.as_ptr());
            if h_mod == 0 {
                return verses;
            }

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
                let key = match self
                    .sword_ptr_to_string(org_crosswire_sword_SWModule_getKeyText(h_mod))
                {
                    Some(k) => k,
                    None => break,
                };
                if !key.starts_with(&chapter_boundary) {
                    break;
                }

                let raw_osis = match self
                    .sword_ptr_to_string(org_crosswire_sword_SWModule_getRawEntry(h_mod))
                {
                    Some(s) => s,
                    None => break,
                };

                // Debug print raw OSIS
                println!("[+] {}\n", raw_osis);

                let parsed_verses = osis_engine.parse_osis_to_sections(&raw_osis, Some(key));
                verses.extend(parsed_verses);

                org_crosswire_sword_SWModule_next(h_mod);
                if org_crosswire_sword_SWModule_popError(h_mod) != 0 {
                    break;
                }
            }
        }
        verses
    }

    unsafe fn sword_ptr_to_string(&self, ptr: *const std::os::raw::c_char) -> Option<String> {
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() })
    }
}
