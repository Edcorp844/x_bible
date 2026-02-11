use crate::app::AppModel;
use relm4::RelmApp;

mod app;
mod features;

#[allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
mod sword_sys {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

const STYLE_CSS: &str = include_str!("../data/style/style.css");

fn main() {
    let app = RelmApp::new("org.flame.xbible");

    gtk::gio::resources_register_include!("xbible.gresource").expect("Failed to register main resources");
    gtk::gio::resources_register_include!("icons.gresource").expect("Failed to register icon resources");

    let provider = gtk::CssProvider::new();
    provider.load_from_string(STYLE_CSS);
    
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Failed to get default display"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let display = gtk::gdk::Display::default().unwrap();
    let theme = gtk::IconTheme::for_display(&display);
    theme.add_resource_path("/com/example/xbible/icons");

    app.run::<AppModel>(());
}