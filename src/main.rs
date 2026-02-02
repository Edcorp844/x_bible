use crate::app::AppModel;
use relm4::RelmApp;

mod app;
mod features;

mod sword_sys {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}
fn main() {
    let app = RelmApp::new("org.flame.xbible");

    // 1. Register both resource files
    // Ensure "xbible.gresource" and "icons.gresource" are in your root/build folder
    gtk::gio::resources_register_include!("xbible.gresource").expect("Main resources failed");
    gtk::gio::resources_register_include!("icons.gresource").expect("Icon resources failed");

    // 2. Load CSS
    relm4::set_global_css_from_file("data/style/style.css").expect("CSS failed");

    // 3. Register the icon path into the Icon Theme
    // Crucial: The path here must match the "prefix" in your XML!
    let display = gtk::gdk::Display::default().unwrap();
    let theme = gtk::IconTheme::for_display(&display);

    // If your XML prefix is "/com/example/xbible/icons/24x24/actions/"
    // You should add the base path where GTK can search
    theme.add_resource_path("/com/example/xbible/icons");

    app.run::<AppModel>(());
}
