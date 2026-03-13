use std::fmt::Debug;

use crate::features::{
    bible::components::page::{
        helpers::{AddedWordStyle, AvailableFonts, PageDisplayConfig},
        verse::VerseInputMessage,
    },
    core::{
        core::AppSetting::AppSetting, display_configurations::display_configuration::DisplayConfig,
    },
};
use gtk::gio::prelude::SettingsExt;
use quick_xml::reader::Config;

impl AppSetting for PageDisplayConfig {
    fn schema_id() -> &'static str {
        "org.flame.xbible.display"
    }
}

impl PageDisplayConfig {
    pub fn new() -> Self {
        Self {
            settings: Self::get_settings(),
        }
    }

    //-----SETTERS-----------

    pub fn set_show_strongs(&self, value: bool) {
        let _ = self.settings.set_boolean("show-strongs", value);
    }

    pub fn set_show_morphs(&self, value: bool) {
        let _ = self.settings.set_boolean("show-morphs", value);
    }

    pub fn set_show_lemma(&self, value: bool) {
        let _ = self.settings.set_boolean("show-lemma", value);
    }

    pub fn set_show_notes(&self, value: bool) {
        let _ = self.settings.set_boolean("show-notes", value);
    }

    pub fn set_added_style(&self, style: AddedWordStyle) {
        // Uses the Display trait we implemented for AddedWordStyle
        let _ = self.settings.set_string("added-style", &style.to_string());
    }
}

impl Debug for dyn DisplayConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(pango_text_size: {}, pango_word_spacing: {}, pango_line_spacing: {}, font:{:?},  font_size: {}, line_spacing: {}, word_spacing: {}, bold_font: {}, show_strongs: {}, show_morphs: {}, show_lemma: {}, show_notes: {}, added_style: {:?})",
            self.pango_text_size(),
            self.pango_word_spacing(),
            self.pango_line_spacing(),
            self.font(),
            self.font_size(),
            self.line_spacing(),
            self.word_spacing(),
            self.bold_font(),
            self.show_strongs(),
            self.show_morphs(),
            self.show_lemma(),
            self.show_notes(),
            self.added_style()
        )
    }
}

impl DisplayConfig for PageDisplayConfig {
    // --- GETTERS ---

    fn font_size(&self) -> f64 {
        self.settings.double("font-size")
    }

    fn font(&self) -> AvailableFonts {
        let font_str = self.settings.string("font-family");
        AvailableFonts::from_string(&font_str)
    }

    fn line_spacing(&self) -> f64 {
        self.settings.double("line-spacing")
    }

    fn word_spacing(&self) -> f64 {
        self.settings.double("word-spacing")
    }

    fn bold_font(&self) -> bool {
        self.settings.boolean("bold-font")
    }

    fn show_strongs(&self) -> bool {
        self.settings.boolean("show-strongs")
    }

    fn show_morphs(&self) -> bool {
        self.settings.boolean("show-morphs")
    }

    fn show_lemma(&self) -> bool {
        self.settings.boolean("show-lemma")
    }

    fn show_notes(&self) -> bool {
        self.settings.boolean("show-notes")
    }

    fn justify(&self) -> bool {
        self.settings.boolean("justify")
    }

    fn added_style(&self) -> AddedWordStyle {
        let style_str = self.settings.string("added-style");
        AddedWordStyle::from_string(&style_str)
    }

    fn theme(&self) -> String {
        self.settings.string("page-theme").to_string()
    }

    fn christ_words_red(&self) -> bool {
        self.settings.boolean("christ-red-words")
    }

    // --- SETTERS ---

    fn set_font_size(&mut self, value: f64) {
        let _ = self.settings.set_double("font-size", value);
    }

    fn set_font(&mut self, font: AvailableFonts) {
        // Uses the Display trait we implemented for AvailableFonts
        let _ = self.settings.set_string("font-family", &font.to_string());
    }

    fn set_line_spacing(&mut self, value: f64) {
        let _ = self.settings.set_double("line-spacing", value);
    }

    fn set_word_spacing(&mut self, value: f64) {
        let _ = self.settings.set_double("word-spacing", value);
    }

    fn set_bold_font(&mut self, value: bool) {
        let _ = self.settings.set_boolean("bold-font", value);
    }

    fn set_justify(&mut self, value: bool) {
        let _ = self.settings.set_boolean("justify", value);
    }

    fn set_christ_words_red(&mut self, value: bool) {
        let _ = self.settings.set_boolean("christ-red-words", value);
    }

    fn set_theme(&mut self, value: String) {
        let _ = self.settings.set_string("page-theme", &value);
    }

    fn apply_message(&mut self, msg: &VerseInputMessage) {
        match msg {
            VerseInputMessage::EnableStrongs => self.set_show_strongs(true),
            VerseInputMessage::DisableStrongs => self.set_show_strongs(false),
            VerseInputMessage::EnableNotes => self.set_show_notes(true),
            VerseInputMessage::DisableNotes => self.set_show_notes(false),
            VerseInputMessage::EnableMorphs => self.set_show_morphs(true),
            VerseInputMessage::DisableMorphs => self.set_show_morphs(false),
            VerseInputMessage::EnableLemma => self.set_show_lemma(true),
            VerseInputMessage::DisableLemma => self.set_show_lemma(false),
            VerseInputMessage::ChangeFontSize(size) => self.set_font_size(*size),
            VerseInputMessage::ChangeFont(font) => self.set_font(*font),
            VerseInputMessage::ChangeWordSpacing(word_spacing) => {
                self.set_word_spacing(*word_spacing)
            }
            VerseInputMessage::ChangeLineSpacing(line_spacing) => {
                self.set_line_spacing(*line_spacing)
            }
            VerseInputMessage::ChangeJustify(justify) => self.set_justify(*justify),
            VerseInputMessage::ChangeBoldFont(bold_font) => self.set_bold_font(*bold_font),
            VerseInputMessage::PutChristWordsInRed(value)=>self.set_christ_words_red(*value),
            VerseInputMessage::UpdateDisplayConf(config) => {
                let config = config.read().unwrap();
                self.set_line_spacing(config.line_spacing());
                self.set_word_spacing(config.word_spacing());
                self.set_font(config.font());
                self.set_bold_font(config.bold_font());
                self.set_font_size(config.font_size());
                self.set_justify(config.justify());
                self.set_theme(config.theme());
            }
            _ => {}
        }
    }
}
