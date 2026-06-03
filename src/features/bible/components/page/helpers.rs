use std::fmt;

use gtk::gio::Settings;

/// How a segment should be rendered or interpreted
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddedWordStyle {
    Italic,
    Brackets,
}

impl AddedWordStyle {
    /// Converts a string from GSettings back into the Enum
    pub fn from_string(s: &str) -> Self {
        match s {
            "Italic" => Self::Italic,
            "Brackets" => Self::Brackets,
            _ => Self::Brackets,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Brackets => format!("Brackets"),
            Self::Italic => format!("Italic"),
        }
    }

    pub fn all() -> Vec<Self> {
        vec![Self::Italic, Self::Brackets]
    }
}

// This allows to call .to_string() on the enum
impl fmt::Display for AddedWordStyle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Italic => "Italic",
            Self::Brackets => "Brackets",
        };
        write!(f, "{s}")
    }
}

#[derive(Clone, Debug)]
pub struct PageDisplayConfig {
    pub settings: Settings,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AvailableFonts {
    Sans,
    Serif,
    Monospace,
    TimesNewRoman,
    System,
}

impl AvailableFonts {
    pub fn all() -> Vec<Self> {
        vec![
            Self::System,
            Self::Sans,
            Self::Serif,
            Self::Monospace,
            Self::TimesNewRoman,
        ]
    }

    pub fn from_string(s: &str) -> Self {
        match s {
            "System" => Self::System,
            "Sans" => Self::Sans,
            "Serif" => Self::Serif,
            "Monospace" => Self::Monospace,
            "Times New Roman" => Self::TimesNewRoman,
            _ => Self::System,
        }
    }
}

impl fmt::Display for AvailableFonts {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::System => "System",
            Self::Sans => "Sans",
            Self::Serif => "Serif",
            Self::Monospace => "Monospace",
            Self::TimesNewRoman => "Times New Roman",
        };
        write!(f, "{s}")
    }
}
