pub mod app_menu;
mod app_settings;
pub mod components;
pub mod display_configurations;
pub mod module_engine;
pub mod osis_translation_engine;
pub mod pages;

pub mod core {
    pub use crate::features::core::app_settings::app_settings::AppSetting;
}
