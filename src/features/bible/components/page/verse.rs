use adw::prelude::*;
use relm4::prelude::*;

use crate::features::{
    bible::components::page::{
        helpers::{AvailableFonts, Verse},
        word::{WordModel, WordModelInput, WordModelOutput},
    },
    core::display_configurations::Config::TextConfig,
};

pub struct VerseModel {
    pub data: Verse,
    pub config: TextConfig,
    pub word_controllers: Vec<Controller<WordModel>>,
    pub text_direction: gtk::TextDirection,
}

#[derive(Debug, Clone)]
pub enum VerseInputMessage {
    UpdateDisplayConf(TextConfig),
    EnableStrongs,
    DisableStrongs,
    EnableNotes,
    DisableNotes,
    EnableMorphs,
    DisableMorphs,
    EnableLemma,
    DisableLemma,
    ChangeFontSize(f64),
    ChangeFont(AvailableFonts),
    ChangeWordSpacing(f64),
    ChangeLineSpacing(f64),
    ChangeBoldFont(bool),
    ChangeJustify(bool),
    PutChristWordsInRed(bool),
    LookUp(String),
}

#[derive(Debug, Clone)]
pub enum VerseOutputMessage {
    Lookup(String),
}

// --- VERSE FACTORY ---
#[relm4::component(pub)]
impl SimpleComponent for VerseModel {
    type Init = (Verse, TextConfig, gtk::TextDirection);
    type Input = VerseInputMessage;
    type Output = VerseOutputMessage;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 12,
            set_hexpand: true,
            add_css_class: "verse-root",

            // 1. Verse Number - Aligned to the top to stay fixed
            gtk::Label {
                add_css_class: "verser-number",
                set_visible: model.data.number != 0,
                set_markup: &format!(
                    "<span size='large'>{}</span>",
                    model.data.number
                ),
                set_valign: gtk::Align::Start,
                set_visible: model.text_direction == gtk::TextDirection::Ltr,
            },

            // 2. Content Stack (Text + Notes)
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 8, // Tight spacing between text and notes
                set_hexpand: true,

                // Main Bible Text
                #[local_ref]
                word_flow -> adw::WrapBox {
                    #[watch]
                    set_line_spacing: model.config.read().unwrap().pango_line_spacing(),
                    set_hexpand: true,
                    #[watch]
                    set_justify: if model.config.read().unwrap().justify() {adw::JustifyMode::Fill} else{adw::JustifyMode::None},
                    set_halign: gtk::Align::Start,
                    #[watch]
                    set_direction: model.text_direction,
                },

                // Notes Revealer - Animates expansion when show_notes is true
                gtk::Revealer {
                    set_transition_type: gtk::RevealerTransitionType::SlideDown,
                    set_transition_duration: 350,

                    // Triggers the slide animation
                    #[watch]
                    set_reveal_child: !model.data.notes.is_empty() && model.config.read().unwrap().show_notes(),

                    // Ensures the revealer doesn't block layout when hidden
                    #[watch]
                    set_visible: !model.data.notes.is_empty(),

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
            },

             gtk::Label {
                add_css_class: "verser-number",
                set_visible: model.data.number != 0,
                set_markup: &format!(
                    "<span size='large'>{}</span>",
                    model.data.number
                ),
                set_valign: gtk::Align::Start,
                set_visible: model.text_direction == gtk::TextDirection::Rtl,
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (data, config, text_direction) = init;

        let mut model = Self {
            data,
            config,
            word_controllers: Vec::new(),
            text_direction,
        };

        let mut word_controllers = Vec::new();
        let word_flow_box = adw::WrapBox::builder()
            .line_spacing(6)
            .hexpand(true)
            .halign(gtk::Align::Start)
            .build();

        for word in &model.data.words {
            let controller = WordModel::builder()
                .launch((word.clone(), model.config.clone(), model.text_direction))
                .forward(sender.input_sender(), move |message| match message {
                    WordModelOutput::LookUp(text) => VerseInputMessage::LookUp(text),
                });

            word_flow_box.append(controller.widget());
            word_controllers.push(controller);
        }

        model.word_controllers = word_controllers;

        let word_flow = &word_flow_box;
        let notes_container = adw::WrapBox::builder()
            .line_spacing(6)
            .hexpand(true)
            .halign(gtk::Align::Start)
            .build();
        for word in model.data.notes.clone() {
            let note_label = gtk::Label::builder()
                .wrap(true)
                .margin_end(16)
                .css_classes(vec!["verse-note"])
                .build();
            note_label.set_markup(
                format!("<span size='large' foreground='#d71452'><span size='small' foreground='#c314d7'><i>Note on Verse {}: </i></span><i>{}</i></span>",model.data.number, word).as_str(),
            );

            notes_container.append(&note_label);
        }

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            VerseInputMessage::LookUp(text) => {
                let _ = sender.output(VerseOutputMessage::Lookup(text));
            }
            _ => {
                self.config.write().unwrap().apply_message(&message);

                for controller in &self.word_controllers {
                    controller.emit(WordModelInput::UpdateConfig(self.config.clone()));
                }
            }
        }
    }
}
