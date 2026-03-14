use crate::features::bible::components::page::{
    helpers::{AddedWordStyle, AvailableFonts},
    verse_components::verse::VerseInputMessage,
};

pub trait DisplayConfig {
    //-----Utilities----

    fn pango_text_size(&self) -> i32 {
        (self.font_size() * 1024.0) as i32
    }

    fn pango_word_spacing(&self) -> i32 {
        let scale_factor = 1.2;
        let spacing = (self.font_size() * 0.60 * scale_factor + self.word_spacing()) as i32 - 10;

        spacing
    }

    fn pango_line_spacing(&self) -> i32 {
        let scale_factor = 8.0;
        (self.line_spacing() * scale_factor) as i32
    }

    // ----- GETTERS --------

    fn font_size(&self) -> f64;

    fn font(&self) -> AvailableFonts;

    fn line_spacing(&self) -> f64;

    fn word_spacing(&self) -> f64;

    fn bold_font(&self) -> bool;

    fn show_strongs(&self) -> bool;

    fn show_morphs(&self) -> bool;

    fn show_lemma(&self) -> bool;

    fn show_notes(&self) -> bool;

    fn justify(&self) -> bool;

    fn theme(&self) -> String;

    fn added_style(&self) -> AddedWordStyle;

    fn christ_words_red(&self) -> bool;

    // -------SETTERS----------

    fn set_font_size(&mut self, value: f64);

    fn set_font(&mut self, font: AvailableFonts);

    fn set_line_spacing(&mut self, value: f64);

    fn set_word_spacing(&mut self, value: f64);

    fn set_bold_font(&mut self, value: bool);

    fn set_justify(&mut self, value: bool);

    fn set_theme(&mut self, value: String);

    fn set_christ_words_red(&mut self, value: bool);

    fn apply_message(&mut self, msg: &VerseInputMessage);
}
