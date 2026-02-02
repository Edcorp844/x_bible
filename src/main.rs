use crate::app::AppModel;
use relm4::RelmApp;

mod app;
mod features;

mod sword_sys {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

fn main() {
    let app = RelmApp::new("org.flame.xbible");
    // init_sword();

    gtk::gio::resources_register_include!("xbible.gresource").expect("Resources failed");
    relm4::set_global_css_from_file("resources/style/style.css").expect("CSS failed");

    app.run::<AppModel>(());
}
