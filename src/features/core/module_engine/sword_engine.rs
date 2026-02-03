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
        let h_module =
            org_crosswire_sword_SWMgr_getModuleByName(self.mgr, c_mod_name.as_ptr());
        if h_module == 0 {
            return books;
        }

        org_crosswire_sword_SWModule_begin(h_module);

        let mut current_book: Option<String> = None;
        let mut chapters: Vec<ModuleChapter> = Vec::new();
        let mut current_chapter: i32 = 0;
        let mut verse_count: usize = 0;

        loop {
            if org_crosswire_sword_SWModule_popError(h_module) != 0 {
                break;
            }

            let key_ptr = org_crosswire_sword_SWModule_getKeyText(h_module);
            if key_ptr.is_null() {
                break;
            }

            let key = CStr::from_ptr(key_ptr).to_string_lossy();

            // Expected: "BookName Chapter:Verse"
            let mut parts = key.split_whitespace();

            let book = match parts.next() {
                Some(b) => b.to_string(),
                None => {
                    org_crosswire_sword_SWModule_next(h_module);
                    continue;
                }
            };

            let chap_verse = match parts.next() {
                Some(cv) => cv,
                None => {
                    org_crosswire_sword_SWModule_next(h_module);
                    continue;
                }
            };

            let chapter: i32 = match chap_verse.split(':').next().unwrap().parse() {
                Ok(c) => c,
                Err(_) => {
                    org_crosswire_sword_SWModule_next(h_module);
                    continue;
                }
            };

            // ---- Book transition ----
            if current_book.as_deref() != Some(&book) {
                if let Some(prev) = current_book.take() {
                    if verse_count > 0 {
                        chapters.push(ModuleChapter {
                            number: current_chapter,
                            verse_count: verse_count as i32,
                        });
                    }

                    books.push(ModuleBook {
                        name: prev,
                        chapters: chapters.clone(),
                    });
                }

                current_book = Some(book);
                chapters.clear();
                current_chapter = 0;
                verse_count = 0;
            }

            // ---- Chapter transition ----
            if chapter != current_chapter {
                if verse_count > 0 {
                    chapters.push(ModuleChapter {
                        number: current_chapter,
                        verse_count: verse_count as i32,
                    });
                }

                current_chapter = chapter;
                verse_count = 0;
            }

            verse_count += 1;
            org_crosswire_sword_SWModule_next(h_module);
        }

        // Flush last book
        if let Some(book) = current_book {
            if verse_count > 0 {
                chapters.push(ModuleChapter {
                    number: current_chapter,
                    verse_count: verse_count as i32,
                });
            }

            books.push(ModuleBook {
                name: book,
                chapters,
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
