use directories::ProjectDirs;
use std::ffi::{CStr, CString};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use crate::features::core::module_engine::sword_module::{ModuleBook, ModuleChapter, SwordModule};
use crate::sword_sys::*;

static PROGRESS_TOTAL: AtomicU64 = AtomicU64::new(0);
static PROGRESS_COMPLETED: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub struct SwordInner {
    pub mgr: isize,
    install_mgr: isize,
}

#[derive(Debug)]
pub struct SwordEngine {
    pub inner: Mutex<SwordInner>,
    sword_path: PathBuf,
}

impl SwordEngine {
    pub fn new() -> Arc<Self> {
        let path = Self::get_sword_path();
        Self::prepare_app_directory(&path);

        let path_str = path.to_string_lossy().replace("\\", "/");
        let c_path = CString::new(path_str.clone()).unwrap();

        unsafe {
            println!("[SwordEngine] Initializing InstallMgr...");
            let install_mgr =
                org_crosswire_sword_InstallMgr_new(c_path.as_ptr(), Some(Self::status_reporter));

            org_crosswire_sword_InstallMgr_setUserDisclaimerConfirmed(install_mgr);
            org_crosswire_sword_InstallMgr_syncConfig(install_mgr);

            println!("[SwordEngine] Initializing SWMgr...");
            let mgr = org_crosswire_sword_SWMgr_newWithPath(c_path.as_ptr());
            let utf8_key = CString::new("UTF8").unwrap();
            let on_val = CString::new("true").unwrap();
            org_crosswire_sword_SWMgr_setGlobalOption(mgr, utf8_key.as_ptr(), on_val.as_ptr());

            println!("[SwordEngine] Initialization complete");

            Arc::new(Self {
                inner: Mutex::new(SwordInner { mgr, install_mgr }),
                sword_path: path,
            })
        }
    }

    unsafe extern "C" fn status_reporter(
        msg: *const ::std::os::raw::c_char,
        total: ::std::os::raw::c_ulong,
        completed: ::std::os::raw::c_ulong,
    ) {
        PROGRESS_TOTAL.store(total as u64, Ordering::SeqCst);
        PROGRESS_COMPLETED.store(completed as u64, Ordering::SeqCst);

        if !msg.is_null() {
            let message = CStr::from_ptr(msg).to_string_lossy();
            println!(
                "[SwordEngine] Progress: {}/{} - {}",
                completed, total, message
            );
        }
    }

    unsafe fn rebuild_mgr(&self, inner: &mut SwordInner) {
        println!("[SwordEngine] Rebuilding SWMgr...");
        org_crosswire_sword_SWMgr_delete(inner.mgr);

        let path_str = self.sword_path.to_string_lossy().replace("\\", "/");
        let c_path = CString::new(path_str).unwrap();

        inner.mgr = org_crosswire_sword_SWMgr_newWithPath(c_path.as_ptr());

        let utf8_key = CString::new("UTF8").unwrap();
        let on_val = CString::new("true").unwrap();
        org_crosswire_sword_SWMgr_setGlobalOption(inner.mgr, utf8_key.as_ptr(), on_val.as_ptr());
        println!("[SwordEngine] SWMgr rebuilt successfully");
    }

    // ------------------- REMOTE SOURCES -------------------

    pub fn get_remote_source_list(&self) -> Vec<String> {
        let inner = self.inner.lock().unwrap();
        let mut sources = Vec::new();
        unsafe {
            let ptr = org_crosswire_sword_InstallMgr_getRemoteSources(inner.install_mgr);
            if !ptr.is_null() {
                let mut i = 0;
                while !(*ptr.offset(i)).is_null() {
                    sources.push(self.ptr_to_str(*ptr.offset(i)));
                    i += 1;
                }
            }
        }
        println!("[SwordEngine] Remote sources: {:?}", sources);
        sources
    }

    pub fn fetch_remote_modules(&self, source_name: &str) -> Vec<SwordModule> {
        println!("\n[Step 1] Locking Engine Inner Mutex...");
        let mut inner = self.inner.lock().unwrap();

        let mut modules = Vec::new();
        let c_source = CString::new(source_name).unwrap();

        unsafe {
            println!("[Step 2] Confirming User Disclaimer for InstallMgr...");
            org_crosswire_sword_InstallMgr_setUserDisclaimerConfirmed(inner.install_mgr);

            println!("[Step 3] Refreshing Remote Source: {}...", source_name);
            let refresh_result = org_crosswire_sword_InstallMgr_refreshRemoteSource(
                inner.install_mgr,
                c_source.as_ptr(),
            );
            println!("[Step 3.1] Refresh result code: {}", refresh_result);

            println!("[Step 4] Syncing InstallMgr Config...");
            org_crosswire_sword_InstallMgr_syncConfig(inner.install_mgr);

            println!("[Step 5] Rebuilding SWMgr to recognize new remote files...");
            self.rebuild_mgr(&mut inner);

            println!(
                "[Step 6] Attempting to retrieve module list for '{}'...",
                source_name
            );
            let mut info_ptr = org_crosswire_sword_InstallMgr_getRemoteModInfoList(
                inner.install_mgr,
                0, // All categories
                c_source.as_ptr(),
            );

            // FALLBACK LOGIC
            if info_ptr.is_null() {
                println!(
                    "[Step 6.1] Named source returned NULL. Attempting Global Fetch (ptr::null)..."
                );
                info_ptr = org_crosswire_sword_InstallMgr_getRemoteModInfoList(
                    inner.install_mgr,
                    0,
                    std::ptr::null(), // Requesting all modules from all sources
                );
            }

            if info_ptr.is_null() {
                println!(
                    "[Step 7] Critical: getRemoteModInfoList still returned NULL. No data found."
                );
                return modules;
            }

            println!("[Step 8] Pointer valid. Iterating through module info structures...");
            let mut i = 0;
            loop {
                let entry = info_ptr.offset(i);

                // Safety check: is the pointer itself or the name field null?
                if entry.is_null() || (*entry).name.is_null() {
                    println!("[Step 8.1] Reached end of list at index {}.", i);
                    break;
                }

                let info = *entry;
                let name = self.ptr_to_str(info.name);

                // Log every 10th module to avoid flooding the console, or just log the count
                if i % 20 == 0 {
                    println!("[Step 8.2] Parsing module: {}", name);
                }

                modules.push(SwordModule {
                    name,
                    description: self.ptr_to_str(info.description),
                    category: self.ptr_to_str(info.category),
                    language: self.ptr_to_str(info.language),
                });

                i += 1;
            }

            println!("[Step 9] Successfully processed {} modules.", modules.len());
        }

        modules
    }
    // ------------------- LOCAL MODULES -------------------

    pub fn get_modules(&self) -> Vec<SwordModule> {
        let mut modules = Vec::new();
        let inner = self.inner.lock().unwrap();
        unsafe {
            let mut ptr = org_crosswire_sword_SWMgr_getModInfoList(inner.mgr);
            while !ptr.is_null() && !(*ptr).name.is_null() {
                let info = *ptr;
                modules.push(SwordModule {
                    name: self.ptr_to_str(info.name),
                    description: self.ptr_to_str(info.description),
                    category: self.ptr_to_str(info.category),
                    language: self.ptr_to_str(info.language),
                });
                ptr = ptr.offset(1);
            }
        }
        println!("[SwordEngine] Local modules: {}", modules.len());
        modules
    }

    pub fn get_modules_by_category(&self, categories: Vec<&str>) -> Vec<SwordModule> {
        self.get_modules()
            .into_iter()
            .filter(|m| categories.contains(&m.category.as_str()))
            .collect()
    }

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

    // ------------------- INSTALL MODULE -------------------

    pub fn install_remote_module(&self, source: &str, module_name: &str) -> i32 {
        let inner = self.inner.lock().unwrap();
        let c_source = CString::new(source).unwrap();
        let c_mod = CString::new(module_name).unwrap();

        PROGRESS_TOTAL.store(0, Ordering::SeqCst);
        PROGRESS_COMPLETED.store(0, Ordering::SeqCst);

        unsafe {
            println!(
                "[SwordEngine] Installing module '{}' from '{}'",
                module_name, source
            );
            org_crosswire_sword_InstallMgr_setUserDisclaimerConfirmed(inner.install_mgr);
            let res = org_crosswire_sword_InstallMgr_remoteInstallModule(
                inner.install_mgr,
                inner.mgr,
                c_source.as_ptr(),
                c_mod.as_ptr(),
            );
            println!("[SwordEngine] Install result: {}", res);
            res
        }
    }

    pub fn get_download_progress(&self) -> f64 {
        let total = PROGRESS_TOTAL.load(Ordering::SeqCst);
        let completed = PROGRESS_COMPLETED.load(Ordering::SeqCst);
        if total == 0 {
            0.0
        } else {
            (completed as f64 / total as f64).clamp(0.0, 1.0)
        }
    }

    // ------------------- BIBLE STRUCTURE -------------------

    pub fn get_bible_structure(&self, module_name: &str) -> Vec<ModuleBook> {
        let mut books = Vec::new();
        let c_mod_name = CString::new(module_name).unwrap();
        let inner = self.inner.lock().unwrap();

        unsafe {
            let h_module =
                org_crosswire_sword_SWMgr_getModuleByName(inner.mgr, c_mod_name.as_ptr());
            if h_module == 0 {
                println!("[SwordEngine] Module '{}' not found", module_name);
                return books;
            }

            org_crosswire_sword_SWModule_begin(h_module);

            let mut current_book: Option<String> = None;
            let mut chapters = Vec::new();
            let mut current_chapter = 0;
            let mut verse_count = 0;

            loop {
                if org_crosswire_sword_SWModule_popError(h_module) != 0 {
                    break;
                }

                let key_ptr = org_crosswire_sword_SWModule_getKeyText(h_module);
                if key_ptr.is_null() {
                    break;
                }

                let key = CStr::from_ptr(key_ptr).to_string_lossy();
                let parts: Vec<&str> = key.split_whitespace().collect();
                if parts.len() < 2 {
                    org_crosswire_sword_SWModule_next(h_module);
                    continue;
                }

                let chap_part = parts.last().unwrap();
                let book_part = parts[..parts.len() - 1].join(" ");
                let chapter: i32 = chap_part
                    .split(':')
                    .next()
                    .and_then(|c| c.parse().ok())
                    .unwrap_or(0);

                if current_book.as_deref() != Some(&book_part) {
                    if let Some(prev) = current_book.take() {
                        if verse_count > 0 {
                            chapters.push(ModuleChapter {
                                number: current_chapter,
                                verse_count,
                            });
                        }
                        books.push(ModuleBook {
                            name: prev,
                            chapters: chapters.clone(),
                        });
                    }
                    current_book = Some(book_part);
                    chapters.clear();
                    current_chapter = chapter;
                    verse_count = 0;
                }

                if chapter != current_chapter {
                    if verse_count > 0 {
                        chapters.push(ModuleChapter {
                            number: current_chapter,
                            verse_count,
                        });
                    }
                    current_chapter = chapter;
                    verse_count = 0;
                }

                verse_count += 1;
                org_crosswire_sword_SWModule_next(h_module);
            }

            if let Some(last) = current_book {
                if verse_count > 0 {
                    chapters.push(ModuleChapter {
                        number: current_chapter,
                        verse_count,
                    });
                }
                books.push(ModuleBook {
                    name: last,
                    chapters,
                });
            }
        }

        books
    }

    // ------------------- HELPERS -------------------

    fn ptr_to_str(&self, ptr: *const i8) -> String {
        if ptr.is_null() {
            "Unknown".to_string()
        } else {
            unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() }
        }
    }

    fn get_sword_path() -> PathBuf {
        let proj_dirs = ProjectDirs::from("org", "flame", "xbible").expect("Path error");
        let path = proj_dirs.data_local_dir().to_path_buf();
        fs::create_dir_all(&path).ok();
        path
    }

    fn prepare_app_directory(path: &PathBuf) {
        let _ = fs::create_dir_all(path.join("mods.d"));
        let _ = fs::create_dir_all(path.join("InstallMgr"));
        let conf_path = path.join("sword.conf");
        if !conf_path.exists() {
            if let Ok(mut file) = fs::File::create(conf_path) {
                let config = r#"[Globals]
DataPath=./
[Install]
Disclaimer=Confirmed
[Repos]
[Remote:CrossWire]
Description=CrossWire HTTP
Protocol=HTTP
Source=www.crosswire.org
Directory=/ftpmirror/pub/sword/raw
"#;
                let _ = writeln!(file, "{}", config);
            }
        }
    }
}

impl Drop for SwordEngine {
    fn drop(&mut self) {
        let inner = self.inner.lock().unwrap();
        unsafe {
            println!("[SwordEngine] Dropping SWMgr and InstallMgr...");
            org_crosswire_sword_InstallMgr_delete(inner.install_mgr);
            org_crosswire_sword_SWMgr_delete(inner.mgr);
            println!("[SwordEngine] Dropped successfully");
        }
    }
}
