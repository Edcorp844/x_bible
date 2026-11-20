use adw::prelude::*;
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
    let shared_engine = init_shared_engine().expect("failed to initialize engine");

    let schema_dir = env!("COMPILED_SCHEMA_DIR");
    unsafe { std::env::set_var("GSETTINGS_SCHEMA_DIR", schema_dir) };

    let app = RelmApp::new("io.github.edcorp844.xbible");

    gtk::gio::resources_register_include!("xbible.gresource")
        .expect("Failed to register main resources");
    gtk::gio::resources_register_include!("icons.gresource")
        .expect("Failed to register icon resources");

    let settings = gtk::Settings::default().expect("Could not get default settings");

    let display = gtk::gdk::Display::default().unwrap();
    let theme = gtk::IconTheme::for_display(&display);
    theme.add_resource_path("/com/example/xbible/icons");

    app.run::<XBibleApp>(shared_engine);
}
