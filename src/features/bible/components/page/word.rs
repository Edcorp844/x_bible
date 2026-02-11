use adw::prelude::*;
use relm4::prelude::*;

use crate::features::bible::components::page::{
    helpers::{SegmentStyle, Word},
    verse::DisplayConfig,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddedWordStyle {
    Italic,
    Brackets,
}

pub struct WordModel {
    
}

impl Word {
    pub fn build_widget(&self, added_style: AddedWordStyle, config: DisplayConfig) -> gtk::Widget {
        let wrapper = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(2)
            .halign(gtk::Align::Start)
            .build();

        let word_wrapper = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(2)
            .halign(gtk::Align::Start)
            .build();

        let label = gtk::Label::builder()
            .use_markup(true)
            .hexpand(false)
            .margin_start(if self.is_punctuation { 0 } else { 8 })
            .css_classes(["bible-text"])
            .xalign(0.0)
            .build();

        label.set_markup(&self.render_word(added_style));
        word_wrapper.append(&label);

        wrapper.append(&word_wrapper);

        if let Some(note_content) = &self.note {
            let note_label = gtk::Label::builder()
                .use_markup(true)
                .hexpand(false)
                .xalign(0.0)
                .build();

            note_label.set_markup("<span color='#d71452' size='x-small'><i>n*</i></span>");

            // --- CURSOR FIX ---
            let motion = gtk::EventControllerMotion::new();
            motion.connect_enter(|motion, _, _| {
                let widget = motion.widget().unwrap();
                // Set the cursor to "pointer" (the hand icon)
                widget.set_cursor_from_name(Some("pointer"));
            });

            motion.connect_leave(|motion| {
                let widget = motion.widget().unwrap();
                // Reset the cursor when leaving
                widget.set_cursor(None);
            });

            note_label.add_controller(motion);

            // --- POPUP LOGIC ---
            let popover = gtk::Popover::builder()
                .css_classes(["bible-note-popover"])
                .autohide(true)
                .build();

            let popover_label = gtk::Label::builder()
                .wrap(true)
                .max_width_chars(30)
                .margin_top(5)
                .margin_bottom(5)
                .margin_start(5)
                .margin_end(5)
                .build();

            popover_label
                .set_markup(format!("<span color='#d71452'>{note_content}</span>").as_str());
            popover.set_child(Some(&popover_label));
            popover.set_parent(&note_label);

            let click = gtk::GestureClick::new();
            let p_clone = popover.clone();
            click.connect_released(move |_, _, _, _| {
                p_clone.popup();
            });

            note_label.add_controller(click);
            word_wrapper.append(&note_label);
        }

        if let Some(lex) = self.lex.as_ref() {
            if config.show_strongs && !lex.strongs.is_empty() {
                let strong_label = gtk::Label::builder()
                    .use_markup(true)
                    .hexpand(false)
                    .css_classes(["bible-text", "lexical"])
                    .xalign(0.0)
                    .margin_end(8)
                    .margin_start(4)
                    .build();

                let joined = lex
                    .strongs
                    .iter()
                    .map(|s| format!("<span size='small' color='#1086ed'>{}</span>", s))
                    .collect::<Vec<_>>()
                    .join(" ");

                strong_label.set_markup(&joined);
                wrapper.append(&strong_label);
                wrapper.add_css_class("word-wrapper");
            }

            if let Some(lemma) = lex.lemma.clone() {
                let strong_label = gtk::Label::builder()
                    .use_markup(true)
                    .hexpand(false)
                    .css_classes(["bible-text", "lexical"])
                    .xalign(0.0)
                    .margin_end(8)
                    .margin_start(4)
                    .build();

                let cleaned = format!("<span  size='small' color='#ed10a3'>{}</span>", lemma);

                strong_label.set_markup(&cleaned);
                if config.show_lemma {
                    wrapper.append(&strong_label);
                    wrapper.add_css_class("word-wrapper");
                }
            }

            if let Some(morph) = lex.morph.clone() {
                println!("{morph}");
                let strong_label = gtk::Label::builder()
                    .use_markup(true)
                    .hexpand(false)
                    .css_classes(["bible-text", "lexical"])
                    .xalign(0.0)
                    .margin_end(8)
                    .margin_start(4)
                    .build();

                let cleaned = format!("<span  size='small' color='#6110ed'>{}</span>", morph);

                strong_label.set_markup(&cleaned);

                if config.show_morphs {
                    wrapper.append(&strong_label);
                    wrapper.add_css_class("word-wrapper");
                }
            }
        }

        wrapper.upcast()
    }

    fn render_word(&self, added_style: AddedWordStyle) -> String {
        let escaped = gtk::glib::markup_escape_text(&self.text);

        let mut content = match self.style {
            SegmentStyle::Added => match added_style {
                AddedWordStyle::Italic => format!("<i>{}</i>", escaped),
                AddedWordStyle::Brackets => {
                    let open = if self.is_first_in_group { "[" } else { "" };
                    let close = if self.is_last_in_group { "]" } else { "" };
                    format!("{open}{escaped}{close}")
                }
            },
            _ => escaped.to_string(),
        };

        if self.is_red {
            content = format!("<span color='#e01b24'>{}</span>", content);
        }

        if self.is_italic {
            content = format!("<i>{}</i>", content);
        }

        if self.is_bold_text {
            content = format!("<b>{}</b>", content);
        }

        format!("<span size='large'>{}</span>", content)
    }
}

impl Default for Word {
    fn default() -> Self {
        Self {
            text: String::new(),
            lex: None,

            style: SegmentStyle::Plain,
            is_red: false,
            is_italic: false,
            is_bold_text: false,
            is_punctuation: false,

            is_first_in_group: false,
            is_last_in_group: false,
            note: None,
        }
    }
}
