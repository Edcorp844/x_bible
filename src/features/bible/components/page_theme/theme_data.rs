use crate::features::bible::components::page::helpers::AvailableFonts;

pub struct ThemeSettings {
    pub font: AvailableFonts,
    pub line_spacing: f64,
    pub word_spacing: f64,
    pub bold_font: bool,
    pub justify: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemePreset {
    Default,
    Classic,
    Modern,
    Compact,
}

impl ThemePreset {
    pub fn all() -> Vec<Self> {
        vec![Self::Default, Self::Classic, Self::Modern, Self::Compact]
    }

    pub fn get_settings(&self) -> ThemeSettings {
        match self {
            Self::Default => ThemeSettings {
                font: AvailableFonts::System,
                line_spacing: 1.5,
                word_spacing: 10.0,
                bold_font: false,
                justify: false,
            },
            Self::Classic => ThemeSettings {
                font: AvailableFonts::Serif,
                line_spacing: 2.0,
                word_spacing: 13.0,
                bold_font: false,
                justify: true,
            },
            Self::Modern => ThemeSettings {
                font: AvailableFonts::Sans,
                line_spacing: 1.5,
                word_spacing: 8.0,
                bold_font: false,
                justify: false,
            },
            Self::Compact => ThemeSettings {
                font: AvailableFonts::System,
                line_spacing: 1.0,
                word_spacing: 12.0,
                bold_font: true,
                justify: false,
            },
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            ThemePreset::Default => format!("Default"),
            ThemePreset::Classic => format!("Classic"),
            ThemePreset::Modern => format!("Modern"),
            ThemePreset::Compact => format!("Compact"),
        }
    }

    pub fn from_string(string: String) -> Self {
        match string.as_str() {
            "Default" => Self::Default,
            "Classic" => Self::Classic,
            "Modern" => Self::Modern,
            "Compact" => Self::Compact,
            _ => Self::Default,
        }
    }
}
