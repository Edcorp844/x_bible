pub mod app_setting {

    use gtk::gio::Settings;

    pub trait AppSetting {
        /// The unique ID defined in the schema XML
        fn schema_id() -> &'static str;

        /// Helper to get the GSettings object for this specific ID
        fn get_settings() -> Settings {
            Settings::new(Self::schema_id())
        }
    }
}
