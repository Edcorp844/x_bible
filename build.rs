use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // 1. --- COMPILE GTK RESOURCES ---
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
