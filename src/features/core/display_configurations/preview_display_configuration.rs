use crate::features::{
    bible::components::{
        page::{
            helpers::{AddedWordStyle, AvailableFonts},
            verse_components::verse::VerseInputMessage,
        },
        page_theme::theme_data::ThemePreset,
    },
    core::display_configurations::{config::TextConfig, display_configuration::DisplayConfig},
};

pub struct PreviewDisplayConfig {
    font_size: f64,
    font: AvailableFonts,
    line_spacing: f64,
    word_spacing: f64,
    bold_font: bool,
    justify: bool,
    theme: String,
    christ_words_red: bool,
}

impl PreviewDisplayConfig {
    fn new() -> Self {
        Self {
            font_size: 0.0,
            font: AvailableFonts::System,
            line_spacing: 0.0,
            word_spacing: 0.0,
            bold_font: false,
            justify: false,
            theme: "Default".to_string(),
            christ_words_red: false,
        }
    }
    pub fn from_page_config(config: TextConfig) -> Self {
        let config = config.read().unwrap();
        let mut preview_config = Self::new();
        preview_config.set_font(config.font());
        preview_config.set_font_size(config.font_size());
        preview_config.set_bold_font(config.bold_font());
        preview_config.set_line_spacing(config.line_spacing());
        preview_config.set_word_spacing(config.word_spacing());
        preview_config.set_justify(config.justify());
        preview_config.set_theme(config.theme());
        preview_config
    }

    pub fn from_theme(theme: ThemePreset, config: TextConfig) -> Self {
        let config = config.read().unwrap();
        let mut preview_config = Self::new();
        preview_config.set_word_spacing(theme.get_settings().word_spacing);
        preview_config.set_line_spacing(theme.get_settings().line_spacing);
        preview_config.set_justify(theme.get_settings().justify);
        preview_config.set_bold_font(theme.get_settings().bold_font);
        preview_config.set_font(theme.get_settings().font);
        preview_config.set_theme(theme.to_string());
        preview_config.set_font_size(config.font_size());
        preview_config.set_christ_words_red(config.christ_words_red());

        preview_config
    }
}

impl DisplayConfig for PreviewDisplayConfig {
    // --- GETTERS ---

    fn font_size(&self) -> f64 {
        self.font_size
    }

    fn font(&self) -> AvailableFonts {
        self.font
    }

    fn line_spacing(&self) -> f64 {
        self.line_spacing
    }

    fn word_spacing(&self) -> f64 {
        self.word_spacing
    }

    fn bold_font(&self) -> bool {
        self.bold_font
    }

    fn show_strongs(&self) -> bool {
        false
    }

    fn show_morphs(&self) -> bool {
        false
    }

    fn show_lemma(&self) -> bool {
        false
    }

    fn show_notes(&self) -> bool {
        false
    }

    fn justify(&self) -> bool {
        self.justify
    }

    fn added_style(&self) -> AddedWordStyle {
        AddedWordStyle::Brackets
    }

    fn theme(&self) -> String {
        self.theme.clone()
    }

    fn christ_words_red(&self) -> bool {
        self.christ_words_red
    }

    // --- SETTERS ---

    fn set_font_size(&mut self, value: f64) {
        self.font_size = value;
    }

    fn set_font(&mut self, font: AvailableFonts) {
        self.font = font;
    }

    fn set_line_spacing(&mut self, value: f64) {
        self.line_spacing = value
    }

    fn set_word_spacing(&mut self, value: f64) {
        self.word_spacing = value
    }

    fn set_bold_font(&mut self, value: bool) {
        self.bold_font = value;
    }

    fn set_justify(&mut self, value: bool) {
        self.justify = value;
    }

    fn set_theme(&mut self, value: String) {
        self.theme = value;
    }

    fn set_christ_words_red(&mut self, value: bool) {
        self.christ_words_red = value
    }

    fn apply_message(&mut self, msg: &VerseInputMessage) {
        match msg {
            VerseInputMessage::ChangeFontSize(size) => self.set_font_size(*size),
            VerseInputMessage::ChangeFont(font) => self.set_font(*font),
            VerseInputMessage::ChangeWordSpacing(word_spacing) => {
                self.set_word_spacing(*word_spacing)
            }
            VerseInputMessage::ChangeLineSpacing(line_spacing) => {
                self.set_line_spacing(*line_spacing)
            }

            VerseInputMessage::ChangeBoldFont(bold_font) => self.set_bold_font(*bold_font),
            VerseInputMessage::ChangeJustify(justify) => self.set_justify(*justify),
            _ => {}
        }
    }
}
