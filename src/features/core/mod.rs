pub mod app_menu;
mod app_settings;
pub mod components;
pub mod display_configurations;
pub mod pages;

pub mod core {
    pub use crate::features::core::app_settings::app_settings::AppSetting;
}
