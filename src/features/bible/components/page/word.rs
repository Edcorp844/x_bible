use adw::prelude::*;
use relm4::prelude::*;

use crate::features::bible::components::page::helpers::{SegmentStyle, Word};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddedWordStyle {
    Italic,
    Brackets,
}

impl Word {
    pub fn build_widget(&self, added_style: AddedWordStyle) -> gtk::Widget {
        // Main wrapper for each word
        let wrapper = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(2)
            .halign(gtk::Align::Start)
            .build();

       // println!(" {} {:?}", self.text, self.style);

        // The word label
        let label = gtk::Label::builder()
            .use_markup(true)
            .hexpand(false)
            .margin_start(if self.is_punctuation { 0 } else { 8 })
            .css_classes(["bible-text"])
            .xalign(0.0)
            .build();

        label.set_markup(&self.render_word(added_style));
        wrapper.append(&label);

        if let Some(note) = &self.note {
            let strong_label = gtk::Label::builder()
                .use_markup(true)
                .hexpand(false)
                .css_classes(["bible-text", "lexical"])
                .xalign(0.0)
                .margin_end(8)
                .margin_start(4)
                .build();

            let cleaned = format!("<span color='#d71452'>{}</span>", note);

            strong_label.set_markup(&cleaned);
            wrapper.append(&strong_label);
            wrapper.add_css_class("word-wrapper");
        }

        // Optional Strong's reference (lexical info)
        if let Some(lex) = self.lex.as_ref() {
            if !lex.strongs.is_empty() {
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
                wrapper.append(&strong_label);
                wrapper.add_css_class("word-wrapper");
            }

            /*  if let Some(morph) = lex.morph.clone() {
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
                wrapper.append(&strong_label);
                wrapper.add_css_class("word-wrapper");
            }
            */
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
