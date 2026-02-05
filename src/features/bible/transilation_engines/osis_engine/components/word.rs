use adw::prelude::*;
use relm4::prelude::*;

use crate::features::bible::transilation_engines::osis_engine::helpers::{SegmentStyle, Word};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddedWordStyle {
    Italic,
    Brackets,
}

impl Word {
    pub fn build_widget(&self, added_style: AddedWordStyle) -> gtk::Widget {
        // Main wrapper for each word
        let wrapper = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(2)
            .halign(gtk::Align::Start)
            .build();

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
                    .map(|s| format!("<span color='#1086ed'>{}</span>", s))
                    .collect::<Vec<_>>()
                    .join(" ");

                strong_label.set_markup(&joined);
                wrapper.append(&strong_label);
                wrapper.add_css_class("word-wrapper");
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

        format!("<span size='large'>{}</span>", content)
    }
}
