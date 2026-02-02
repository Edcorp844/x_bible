use std::env;
use std::path::PathBuf;

fn main() {
    // 1. --- BUILD THE SWORD ENGINE ---
    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let sword_src = root.join("sword");

    let dst = cmake::Config::new(&sword_src)
        .define("SWORD_BUILD_SHARED", "OFF") // Keep our core engine static
        .define("SWORD_BUILD_EXAMPLES", "OFF")
        .define("SWORD_BUILD_TESTS", "OFF")
        .build();

    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=sword");

    // 2. --- LINK SYSTEM DEPENDENCIES PER OS ---
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    match target_os.as_str() {
        "windows" => {
            // Static link compression to reduce DLL count for the user
            println!("cargo:rustc-link-lib=static=z");
            println!("cargo:rustc-link-lib=static=bz2");
            println!("cargo:rustc-link-lib=static=lzma");

            // Dynamic link networking and system libs
            println!("cargo:rustc-link-lib=dylib=curl");
            println!("cargo:rustc-link-lib=dylib=ws2_32");
            println!("cargo:rustc-link-lib=dylib=crypt32");
            println!("cargo:rustc-link-lib=dylib=stdc++");
        }
        "macos" => {
            // macOS standard paths
            println!("cargo:rustc-link-lib=dylib=curl");
            println!("cargo:rustc-link-lib=dylib=z");
            println!("cargo:rustc-link-lib=dylib=bz2");
            println!("cargo:rustc-link-lib=dylib=lzma");
            println!("cargo:rustc-link-lib=dylib=c++");
            println!("cargo:rustc-link-lib=framework=CoreFoundation");
            println!("cargo:rustc-link-lib=framework=Security");
        }
        _ => {
            // Linux: Try to link statically where possible for portability (AppImage style)
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
        &["resources"],
        "resources/resources.gresource.xml",
        "xbible.gresource",
    );
}
