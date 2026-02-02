use directories::ProjectDirs;
use gtk::StringList;
use std::ffi::{CStr, CString};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use crate::features::core::sword_module::SwordModule;
use crate::sword_sys::*;

pub struct SwordEngine {
    pub mgr: isize,
}

impl SwordEngine {
    pub fn new() -> Self {
        let path = Self::get_sword_path();
        Self::prepare_app_directory(&path);

        // Convert path for the C++ engine (using forward slashes even on Windows)
        let path_str = path.to_string_lossy().replace("\\", "/");
        let c_path = CString::new(path_str).unwrap();

        unsafe {
            let mgr_ptr = org_crosswire_sword_SWMgr_newWithPath(c_path.as_ptr());

            // Force UTF-8 for UI compatibility
            let utf8_key = CString::new("UTF8").unwrap();
            let on_val = CString::new("true").unwrap();
            org_crosswire_sword_SWMgr_setGlobalOption(mgr_ptr, utf8_key.as_ptr(), on_val.as_ptr());

            Self { mgr: mgr_ptr }
        }
    }

    pub fn get_modules(&self) -> Vec<SwordModule> {
        let mut modules = Vec::new();

        unsafe {
            let mut info_ptr = org_crosswire_sword_SWMgr_getModInfoList(self.mgr);

            if info_ptr.is_null() {
                return modules;
            }

            while !info_ptr.is_null() && !(*info_ptr).name.is_null() {
                let info = *info_ptr;

                // Helper to safely convert C strings to Rust Strings
                let to_rust_str = |ptr: *const i8| -> String {
                    if ptr.is_null() {
                        String::from("Unknown")
                    } else {
                        CStr::from_ptr(ptr).to_string_lossy().into_owned()
                    }
                };

                let module = SwordModule {
                    name: to_rust_str(info.name),
                    description: to_rust_str(info.description),
                    category: to_rust_str(info.category),
                    language: to_rust_str(info.language),
                };

                // Filter for Bibles/Texts
                if module.category == "Biblical Texts" || module.category == "Bibles" {
                    modules.push(module);
                }

                info_ptr = info_ptr.offset(1);
            }
        }
        modules
    }

    fn get_sword_path() -> PathBuf {
        let proj_dirs = ProjectDirs::from("org", "flame", "xbible")
            .expect("Could not determine home directory");

        let path = proj_dirs.data_local_dir();

        if !path.exists() {
            std::fs::create_dir_all(&path).ok();
        }

        path.to_path_buf()
    }

    fn prepare_app_directory(path: &PathBuf) {
        if !path.exists() {
            fs::create_dir_all(path).expect("Failed to create App data directory");
        }

        let mods_d = path.join("mods.d");
        if !mods_d.exists() {
            fs::create_dir_all(mods_d).ok();
        }

        let conf_path = path.join("sword.conf");
        if !conf_path.exists() {
            // Because 'use std::io::Write' is at the top, writeln! now works on 'file'
            let mut file = fs::File::create(conf_path).expect("Could not create sword.conf");

            writeln!(file, "[Globals]").ok();
            writeln!(file, "DataPath=./").ok();
        }
    }
}

// Clean up the memory when the app closes
impl Drop for SwordEngine {
    fn drop(&mut self) {
        unsafe {
            org_crosswire_sword_SWMgr_delete(self.mgr);
        }
    }
}
