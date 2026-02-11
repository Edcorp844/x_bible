use adw::prelude::*;
use relm4::{FactorySender, prelude::*};

use crate::features::bible::components::page::{
    helpers::Verse,
    word::{AddedWordStyle, WordModel, WordModelInput},
};

#[derive(Clone, Copy, Debug)]
pub struct DisplayConfig {
    pub show_strongs: bool,
    pub show_morphs: bool,
    pub show_lemma: bool,
    pub show_notes: bool,
    pub added_style: AddedWordStyle,
}

pub struct VerseModel {
    pub data: Verse,           // The "Pure" data struct from your extension
    pub config: DisplayConfig, // The UI-only state

    pub word_controllers: Vec<Controller<WordModel>>,
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
    ChangeFontSize(f64),
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
            add_css_class: "verse-root",

            // 1. Verse Number - Aligned to the top to stay fixed
            gtk::Label {
                add_css_class: "verser-number",
                set_markup: &format!(
                    "<span size='large'>{}</span>",
                    self.data.number
                ),
                set_valign: gtk::Align::Start,
            },

            // 2. Content Stack (Text + Notes)
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 8, // Tight spacing between text and notes
                set_hexpand: true,

                // Main Bible Text
                #[local_ref]
                word_flow -> adw::WrapBox {
                    set_line_spacing: 12,
                    set_hexpand: true,
                    set_halign: gtk::Align::Start,
                },

                // Notes Revealer - Animates expansion when show_notes is true
                gtk::Revealer {
                    set_transition_type: gtk::RevealerTransitionType::SlideDown,
                    set_transition_duration: 350,

                    // Triggers the slide animation
                    #[watch]
                    set_reveal_child: !self.data.notes.is_empty() && self.config.show_notes,

                    // Ensures the revealer doesn't block layout when hidden
                    #[watch]
                    set_visible: !self.data.notes.is_empty(),

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_margin_top: 4,

                        #[local_ref]
                        notes_container -> adw::WrapBox {
                            add_css_class: "verse-notes",
                            set_hexpand: true,
                        }
                    }
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            data: init.0,
            config: init.1,
            word_controllers: Vec::new(),
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        _root: Self::Root,
        _returned_widget: &gtk::Widget,
        _sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let mut word_controllers = Vec::new();
        let word_flow_box = adw::WrapBox::builder()
            .line_spacing(6)
            .hexpand(true)
            .halign(gtk::Align::Start)
            .build();

        for word in &self.data.words {
            let controller = WordModel::builder()
                .launch((word.clone(), self.config.clone()))
                .detach();

            word_flow_box.append(controller.widget());
            word_controllers.push(controller);
        }

        self.word_controllers = word_controllers;

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
            VerseInputMessage::EnableStrongs => {
                self.config.show_strongs = true;
            }
            VerseInputMessage::DisableStrongs => self.config.show_strongs = false,
            VerseInputMessage::EnableNotes => self.config.show_notes = true,
            VerseInputMessage::DisableNotes => self.config.show_notes = false,
            VerseInputMessage::EnableMorphs => self.config.show_morphs = true,
            VerseInputMessage::DisableMorphs => self.config.show_morphs = false,
            VerseInputMessage::EnableLemma => self.config.show_lemma = true,
            VerseInputMessage::DisableLemma => self.config.show_lemma = false,
            VerseInputMessage::ChangeFontSize(font_scale) => {}
        }

        for controller in &self.word_controllers {
            controller.emit(WordModelInput::UpdateConfig(self.config.clone()));
        }
    }
}
