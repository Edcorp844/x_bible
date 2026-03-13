use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // 1. --- BUILD THE SWORD ENGINE ---
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let sword_src = root.join("sword");

    let dst = cmake::Config::new(&sword_src)
        .define("SWORD_BUILD_SHARED", "OFF") // Keep engine static
        .define("SWORD_BUILD_EXAMPLES", "OFF")
        .define("SWORD_BUILD_TESTS", "OFF")
        .build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=sword");

    // 2. --- LINK SYSTEM DEPENDENCIES PER OS ---
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    match target_os.as_str() {
        "windows" => {
            println!("cargo:rustc-link-lib=static=z");
            println!("cargo:rustc-link-lib=static=bz2");
            println!("cargo:rustc-link-lib=static=lzma");
            println!("cargo:rustc-link-lib=dylib=curl");
            println!("cargo:rustc-link-lib=dylib=ws2_32");
            println!("cargo:rustc-link-lib=dylib=crypt32");
            println!("cargo:rustc-link-lib=dylib=stdc++");
        }
        "macos" => {
            println!("cargo:rustc-link-lib=dylib=curl");
            println!("cargo:rustc-link-lib=dylib=z");
            println!("cargo:rustc-link-lib=dylib=bz2");
            println!("cargo:rustc-link-lib=dylib=lzma");
            println!("cargo:rustc-link-lib=dylib=c++");
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=framework=Security");
        }
        _ => {
            // Linux: Use pkg-config to link ICU and other system libs
            let icu_uc = pkg_config::Config::new().probe("icu-uc").unwrap();
            let icu_i18n = pkg_config::Config::new().probe("icu-i18n").unwrap();

            for lib_path in icu_uc.link_paths.iter().chain(icu_i18n.link_paths.iter()) {
                println!("cargo:rustc-link-search=native={}", lib_path.display());
            }
            for lib in icu_uc.libs.iter().chain(icu_i18n.libs.iter()) {
                println!("cargo:rustc-link-lib=dylib={}", lib);
            }

            println!("cargo:rustc-link-lib=dylib=curl");
            println!("cargo:rustc-link-lib=dylib=z");
            println!("cargo:rustc-link-lib=dylib=bz2");
            println!("cargo:rustc-link-lib=dylib=lzma");
            println!("cargo:rustc-link-lib=dylib=stdc++");
        }
    }

    // 3. --- GENERATE BINDINGS ---
    let include_path = dst.join("include");
    let header_path = include_path.join("sword").join("flatapi.h");

    let bindings = bindgen::Builder::default()
        .header(header_path.to_str().expect("Could not find flatapi.h"))
        .clang_arg(format!("-I{}", include_path.display()))
        .allowlist_function("org_crosswire_sword.*")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // 4. --- COMPILE GTK RESOURCES ---
    glib_build_tools::compile_resources(
        &["data"],
        "data/resources.gresource.xml",
        "xbible.gresource",
    );

    glib_build_tools::compile_resources(
        &["data/icons"],
        "data/icons/icons.gresource.xml",
        "icons.gresource",
    );

    // 5. --- COMPILE GSETTINGS SCHEMAS (Clean Industry Standard) ---
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let schema_src = root.join("schemas");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let schema_out_dir = out_path.join("gschemas");

    // We use standard println for success tracking (visible with -v)
    println!("Step 5: GSettings Schema Compilation");
    println!("   Source: {:?}", schema_src);
    println!("   Target: {:?}", schema_out_dir);

    if !schema_src.exists() {
        panic!("\n[STOP] Schema folder missing at: {:?}\n", schema_src);
    }

    fs::create_dir_all(&schema_out_dir).expect("Failed to create schema output directory");

    let mut count = 0;
    if let Ok(entries) = fs::read_dir(&schema_src) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "xml") {
                let name = path.file_name().unwrap();
                fs::copy(&path, schema_out_dir.join(name)).ok();
                println!("   [COPY] {:?}", name);
                count += 1;
            }
        }
    }

    if count == 0 {
        panic!("\n[STOP] No .xml files found in schemas/ folder!\n");
    }

    let status = Command::new("glib-compile-schemas")
        .arg(&schema_out_dir)
        .status()
        .expect("Failed to run glib-compile-schemas");

    if status.success() {
        let final_file = schema_out_dir.join("gschemas.compiled");
        if final_file.exists() {
            println!("   [SUCCESS] Compiled schema created at: {:?}", final_file);
        } else {
            panic!("\n[ERROR] glib-compile-schemas reported success but no file was created.\n");
        }
    } else {
        panic!("\n[ERROR] glib-compile-schemas failed. Check XML syntax.\n");
    }

    // Export the directory so the Rust app knows where to point GSETTINGS_SCHEMA_DIR
    println!(
        "cargo:rustc-env=COMPILED_SCHEMA_DIR={}",
        schema_out_dir.display()
    );
    println!("cargo:rerun-if-changed=schemas");
}
