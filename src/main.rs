use crate::app::AppModel;
use gtk::prelude::*;
use relm4::RelmApp; // Essential for trait methods like .connect_notify_local

mod app;
mod features;

#[allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
mod sword_sys {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

const STYLE_CSS_LIGHT: &str = include_str!("../data/style/light/style.css");
const STYLE_CSS_DARK: &str = include_str!("../data/style/dark/style.css");

fn main() {
    let schema_dir = env!("COMPILED_SCHEMA_DIR");
    unsafe { std::env::set_var("GSETTINGS_SCHEMA_DIR", schema_dir) };

    let app = RelmApp::new("org.flame.xbible");

    gtk::gio::resources_register_include!("xbible.gresource")
        .expect("Failed to register main resources");
    gtk::gio::resources_register_include!("icons.gresource")
        .expect("Failed to register icon resources");

    // --- Theme Management ---
    let provider = gtk::CssProvider::new();
    let settings = gtk::Settings::default().expect("Could not get default settings");

    // Helper function to load the correct CSS based on theme state
    let load_css = {
        let provider = provider.clone();
        let settings = settings.clone();
        move || {
            if settings.is_gtk_application_prefer_dark_theme() {
                provider.load_from_string(STYLE_CSS_DARK);
            } else {
                provider.load_from_string(STYLE_CSS_LIGHT);
            }
        }
    };

    load_css();

    // Listen for theme changes (e.g., system toggle)
    settings.connect_notify_local(Some("gtk-application-prefer-dark-theme"), move |_, _| {
        load_css();
    });

    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Failed to get default display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    // ------------------------

    let display = gtk::gdk::Display::default().unwrap();
    let theme = gtk::IconTheme::for_display(&display);
    theme.add_resource_path("/com/example/xbible/icons");

    app.run::<AppModel>(());
}
