pub mod display_configuration;
pub mod page_display_config;
pub mod preview_display_configuration;
pub mod Config {
    use std::sync::{Arc, RwLock};

    use crate::features::core::display_configurations::display_configuration::DisplayConfig;

    pub type TextConfig = Arc<RwLock<dyn DisplayConfig>>;
}
