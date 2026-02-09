use adw::prelude::*;
use relm4::{FactorySender, prelude::*};

use crate::features::bible::components::page::{helpers::Verse, word::AddedWordStyle};

#[derive(Clone, Copy, Debug)]
pub struct DisplayConfig {
    pub show_strongs: bool,
    pub show_morphs: bool,
    pub show_lemma: bool,
    pub show_notes: bool,
}

pub struct VerseModel {
    pub data: Verse,           // The "Pure" data struct from your extension
    pub config: DisplayConfig, // The UI-only state
}

#[derive(Debug, Clone)]
pub enum VerseInputMessage {
    EnableStrongs,
    DisableStrongs,
    EnableNotes,
    DisableNotes,
    EnableMorphs,
    DisableMorphs,
    EnableLemma,
    DisableLemma,
}

// --- VERSE FACTORY ---
#[relm4::factory(pub)]
impl FactoryComponent for VerseModel {
    type Init = (Verse, DisplayConfig);
    type Input = VerseInputMessage;
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 12,
            set_hexpand: true,

            gtk::Label {
                add_css_class: "verser-number",
                set_markup: &format!(
                    "<span size='large'>{}</span>",
                    self.data.number
                ),
                set_valign: gtk::Align::Start,
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 12,

                #[local_ref]
                word_flow -> adw::WrapBox {
                    set_line_spacing: 12,
                    set_hexpand: true,
                    set_halign: gtk::Align::Start,
                },

                #[local_ref]
                notes_container -> adw::WrapBox{
                    #[watch]
                    set_visible: !self.data.notes.is_empty() && self.config.show_notes,
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            data: init.0,
            config: init.1,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        _root: Self::Root,
        _returned_widget: &gtk::Widget,
        _sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let word_flow_box = adw::WrapBox::builder()
            .line_spacing(6)
            .hexpand(true)
            .halign(gtk::Align::Start)
            .build();

        for word in &self.data.words {
            let word = word.build_widget(AddedWordStyle::Italic, self.config);

            word_flow_box.append(&word);
        }

        let word_flow = &word_flow_box;
        let notes_container = adw::WrapBox::builder()
            .line_spacing(6)
            .hexpand(true)
            .halign(gtk::Align::Start)
            .build();
        for word in self.data.notes.clone() {
            let note_label = gtk::Label::builder()
                .wrap(true)
                .margin_end(16)
                .css_classes(vec!["verse-note"])
                .build();
            note_label.set_markup(
                format!("<span size='large' foreground='#d71452'><span size='small' foreground='#c314d7'><i>Note on Verse {}: </i></span><i>{}</i></span>",self.data.number, word).as_str(),
            );

            notes_container.append(&note_label);
        }
        let widgets = view_output!();

        widgets
    }

    fn update(&mut self, msg: Self::Input, _sender: FactorySender<Self>) {
        match msg {
            VerseInputMessage::EnableStrongs => self.config.show_strongs = true,
            VerseInputMessage::DisableStrongs => self.config.show_strongs = false,
            VerseInputMessage::EnableNotes => self.config.show_notes = true,
            VerseInputMessage::DisableNotes => self.config.show_notes = false,
            VerseInputMessage::EnableMorphs => self.config.show_morphs = true,
            VerseInputMessage::DisableMorphs => self.config.show_morphs = false,
            VerseInputMessage::EnableLemma => self.config.show_lemma = true,
            VerseInputMessage::DisableLemma => self.config.show_lemma = false,
        }
    }
}
