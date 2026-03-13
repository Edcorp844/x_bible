pub mod AppSetting {

    use gtk::gio::Settings;
    use gtk::gio::prelude::*;

    pub trait AppSetting {
        /// The unique ID defined in the schema XML
        fn schema_id() -> &'static str;

        /// Helper to get the GSettings object for this specific ID
        fn get_settings() -> Settings {
            Settings::new(Self::schema_id())
        }
    }
}
