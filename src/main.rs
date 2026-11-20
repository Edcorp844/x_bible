use relm4::RelmApp;

use crate::{app::XBibleApp, core::engine::init_shared_engine};

#[macro_use]
extern crate log;

mod app;
mod core;
mod features;
mod navigation;
mod workers;

fn main() {
    if let Err(err) = run() {
        error!("Application startup failed: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let shared_engine = init_shared_engine()?;

    let schema_dir = env!("COMPILED_SCHEMA_DIR");
    // SAFETY: Called once during single-threaded startup before spawning threads or GTK/Relm4.
    unsafe { std::env::set_var("GSETTINGS_SCHEMA_DIR", schema_dir) };

    let app = RelmApp::new("io.github.edcorp844.xbible");

    if let Err(err) = gtk::gio::resources_register_include!("xbible.gresource") {
        error!("Failed to register main resources: {err}");
    }
    if let Err(err) = gtk::gio::resources_register_include!("icons.gresource") {
        error!("Failed to register icon resources: {err}");
    }

    if gtk::Settings::default().is_none() {
        warn!("Could not get default gtk::Settings");
    }

    match gtk::gdk::Display::default() {
        Some(display) => {
            let theme = gtk::IconTheme::for_display(&display);
            theme.add_resource_path("/com/example/xbible/icons");
        }
        None => {
            warn!("No default GDK display available; skipping icon resource path");
        }
    }

    app.run::<XBibleApp>(shared_engine);
    Ok(())
}
