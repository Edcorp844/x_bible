use directories::ProjectDirs;
use std::ffi::{CStr, CString};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::features::core::module_engine::sword_module::{ModuleBook, ModuleChapter, SwordModule};
use crate::sword_sys::*;

pub struct SwordEngine {
    // PROTECTED: This is a raw pointer to C++ memory.
    // It must NEVER be public so that the UI cannot modify it directly.
    pub mgr: isize,
}

impl SwordEngine {
    /// PUBLIC: The only way to create the engine.
    pub fn new() -> Self {
        let path = Self::get_sword_path();
        Self::prepare_app_directory(&path);

        let path_str = path.to_string_lossy().replace("\\", "/");
        let c_path = CString::new(path_str).unwrap();

        unsafe {
            let mgr_ptr = org_crosswire_sword_SWMgr_newWithPath(c_path.as_ptr());
            let utf8_key = CString::new("UTF8").unwrap();
            let on_val = CString::new("true").unwrap();
            org_crosswire_sword_SWMgr_setGlobalOption(mgr_ptr, utf8_key.as_ptr(), on_val.as_ptr());

            Self { mgr: mgr_ptr }
        }
    }

    // --- Core Data Fetchers (Internal/Crate-Level) ---

    /// PUB(CRATE): Useful for other backend features in this app,
    /// but maybe not something the UI needs to call directly.
    pub(crate) fn get_modules(&self) -> Vec<SwordModule> {
        let mut modules = Vec::new();
        unsafe {
            let mut info_ptr = org_crosswire_sword_SWMgr_getModInfoList(self.mgr);
            if info_ptr.is_null() {
                return modules;
            }

            while !info_ptr.is_null() && !(*info_ptr).name.is_null() {
                let info = *info_ptr;
                let to_rust_str = |ptr: *const i8| -> String {
                    if ptr.is_null() {
                        "Unknown".to_string()
                    } else {
                        CStr::from_ptr(ptr).to_string_lossy().into_owned()
                    }
                };

                modules.push(SwordModule {
                    name: to_rust_str(info.name),
                    description: to_rust_str(info.description),
                    category: to_rust_str(info.category),
                    language: to_rust_str(info.language),
                });
                info_ptr = info_ptr.offset(1);
            }
        }
        modules
    }

    /// PUB(CRATE): A helper for the engine to filter by any string.
    pub(crate) fn get_modules_by_category(&self, categories: Vec<&str>) -> Vec<SwordModule> {
        self.get_modules()
            .into_iter()
            .filter(|m| categories.contains(&m.category.as_str()))
            .collect()
    }

    // --- The Public API (What the UI sees) ---

    pub fn get_bible_modules(&self) -> Vec<SwordModule> {
        self.get_modules_by_category(vec!["Biblical Texts", "Bibles"])
    }

    pub fn get_commentary_modules(&self) -> Vec<SwordModule> {
        self.get_modules_by_category(vec!["Commentaries"])
    }

    pub fn get_dictionary_modules(&self) -> Vec<SwordModule> {
        self.get_modules_by_category(vec!["Lexicons", "Dictionaries"])
    }

    pub fn get_book_modules(&self) -> Vec<SwordModule> {
        self.get_modules_by_category(vec!["Generic Books"])
    }

    pub fn get_map_modules(&self) -> Vec<SwordModule> {
        self.get_modules_by_category(vec!["Images", "Maps"])
    }

    pub fn get_bible_structure(&self, module_name: &str) -> Vec<ModuleBook> {
        let mut books: Vec<ModuleBook> = Vec::new();
        let c_mod_name = CString::new(module_name).unwrap();

        unsafe {
            let h_module = org_crosswire_sword_SWMgr_getModuleByName(self.mgr, c_mod_name.as_ptr());
            if h_module == 0 {
                return books;
            }

            // Reset to start of module
            org_crosswire_sword_SWModule_begin(h_module);

            let mut current_book_name: Option<String> = None;
            let mut chapters_accumulator: Vec<ModuleChapter> = Vec::new();
            let mut current_chapter_num: i32 = 0;
            let mut verse_count: i32 = 0;

            loop {
                // Check for end of module
                if org_crosswire_sword_SWModule_popError(h_module) != 0 {
                    break;
                }

                let key_ptr = org_crosswire_sword_SWModule_getKeyText(h_module);
                if key_ptr.is_null() {
                    break;
                }

                let key = CStr::from_ptr(key_ptr).to_string_lossy();
                let mut parts = key.split_whitespace();

                // Handle books with spaces in names (e.g., "1 John")
                let mut book_part = String::new();
                let mut chap_verse_part = String::new();

                let all_parts: Vec<&str> = key.split_whitespace().collect();
                if all_parts.len() >= 2 {
                    chap_verse_part = all_parts.last().unwrap().to_string();
                    book_part = all_parts[..all_parts.len() - 1].join(" ");
                }

                let chapter: i32 = chap_verse_part
                    .split(':')
                    .next()
                    .and_then(|c| c.parse().ok())
                    .unwrap_or(0);

                // ---- Book Transition Logic ----
                if current_book_name.as_deref() != Some(&book_part) {
                    if let Some(prev_name) = current_book_name.take() {
                        // Push the last chapter of the PREVIOUS book
                        if verse_count > 0 {
                            chapters_accumulator.push(ModuleChapter {
                                number: current_chapter_num,
                                verse_count,
                            });
                        }
                        // Push the PREVIOUS book
                        books.push(ModuleBook {
                            name: prev_name,
                            chapters: chapters_accumulator.clone(),
                        });
                    }
                    current_book_name = Some(book_part);
                    chapters_accumulator.clear();
                    current_chapter_num = chapter;
                    verse_count = 0;
                }

                // ---- Chapter Transition Logic ----
                if chapter != current_chapter_num {
                    if verse_count > 0 {
                        chapters_accumulator.push(ModuleChapter {
                            number: current_chapter_num,
                            verse_count,
                        });
                    }
                    current_chapter_num = chapter;
                    verse_count = 0;
                }

                verse_count += 1;
                org_crosswire_sword_SWModule_next(h_module);
            }

            // ---- THE FINAL FLUSH (Crucial for Revelation) ----
            if let Some(last_book_name) = current_book_name {
                // 1. Push the final chapter (e.g., Revelation 22)
                if verse_count > 0 {
                    chapters_accumulator.push(ModuleChapter {
                        number: current_chapter_num,
                        verse_count,
                    });
                }
                // 2. Push the final book (Revelation)
                books.push(ModuleBook {
                    name: last_book_name,
                    chapters: chapters_accumulator,
                });
            }
        }

        books
    }

    // --- Private Setup Logic ---

    fn get_sword_path() -> PathBuf {
        let proj_dirs = ProjectDirs::from("org", "flame", "xbible")
            .expect("Could not determine home directory");
        let path = proj_dirs.data_local_dir();
        if !path.exists() {
            fs::create_dir_all(&path).ok();
        }
        path.to_path_buf()
    }

    fn prepare_app_directory(path: &PathBuf) {
        let mods_d = path.join("mods.d");
        if !mods_d.exists() {
            fs::create_dir_all(mods_d).ok();
        }

        let conf_path = path.join("sword.conf");
        if !conf_path.exists() {
            let mut file = fs::File::create(conf_path).ok().unwrap();
            writeln!(file, "[Globals]\nDataPath=./").ok();
        }
    }
}

impl Drop for SwordEngine {
    fn drop(&mut self) {
        unsafe {
            org_crosswire_sword_SWMgr_delete(self.mgr);
        }
    }
}
