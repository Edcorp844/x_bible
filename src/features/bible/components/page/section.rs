use gtk::prelude::*;
use relm4::prelude::*;

use crate::features::{
    bible::components::page::{
        helpers::TitleStyle,
        verse_components::{
            verse::{VerseInputMessage, VerseModel, VerseOutputMessage},
            verse_annotation::{Annotations, VerseAnnotation},
        },
        word::{WordModel, WordModelInput},
    },
    core::{
        display_configurations::Config::TextConfig,
        module_engine::{
            sword_engine_dictionary_ext::DictionaryQuery, sword_engine_module_content_ext::Section,
        },
    },
};

pub struct SectionModel {
    pub data: Section,
    pub config: TextConfig,
    pub verses: Vec<Controller<VerseModel>>,
    pub title_word_controllers: Vec<Controller<WordModel>>,
    pub annotations: Annotations,
}

#[derive(Debug, Clone)]
pub enum SectionInput {
    ToggleDisplay(VerseInputMessage),
    Lookup(DictionaryQuery),
}

#[derive(Debug)]
pub enum SectionOutput {
    Lookup(DictionaryQuery),
}

#[relm4::factory(pub)]
impl FactoryComponent for SectionModel {
    type Init = (Section, TextConfig, Annotations);
    type Input = SectionInput;
    type Output = SectionOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 16,
            add_css_class: "section-container",
            #[watch]
            set_margin_bottom: self.config.read().unwrap().pango_line_spacing(),

            // 1. Title Container (Now a WrapBox instead of a Label)
            #[local_ref]
            title_flow -> adw::WrapBox {
                #[watch]
                set_visible: !self.data.title.is_empty(),
                set_line_spacing: 6,
                set_margin_top: self.config.read().unwrap().pango_line_spacing() + 8,
                #[watch]
                set_margin_bottom: self.config.read().unwrap().pango_line_spacing() + 8,
                set_hexpand: false,
                #[watch]
                set_direction: self.data.text_direction.to_gtk_text_direction(),

                // Styling based on H1, H2, etc.
                #[watch]
                add_css_class:"title-3",

            },

            // 2. Verses
            #[local_ref]
            verse_box -> adw::WrapBox {
                set_hexpand: false,
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        let (section_data, config, annotations) = init;

        Self {
            data: section_data,
            config,
            verses: Vec::new(),
            title_word_controllers: Vec::new(),
            annotations,
        }
    }

    fn init_widgets(
        &mut self,
        _index: &DynamicIndex,
        _root: Self::Root,
        _returned_widget: &gtk::Widget,

        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        // Build the Title Words
        let title_flow_box = adw::WrapBox::builder().build();
        let mut title_controllers = Vec::new();

        for word in &self.data.title {
            let controller = WordModel::builder()
                .launch((
                    word.clone(),
                    self.config.clone(),
                    self.data.text_direction.to_gtk_text_direction(),
                    VerseAnnotation::new(),
                ))
                .detach();

            title_flow_box.append(controller.widget());
            title_controllers.push(controller);
        }

        self.title_word_controllers = title_controllers;

        let title_flow = &title_flow_box;

        let verse_box = adw::WrapBox::builder().build();
        let mut verses_controllers = Vec::new();

        for verse in &self.data.verses {
            let annotation = self
                .annotations
                .get(&verse.osis_id)
                .cloned()
                .unwrap_or_default();

            let controller = VerseModel::builder()
                .launch((
                    verse.clone(),
                    self.config.clone(),
                    self.data.text_direction.to_gtk_text_direction(),
                    annotation,
                ))
                .forward(sender.input_sender(), move |message| match message {
                    VerseOutputMessage::Lookup(query) => SectionInput::Lookup(query),
                });
            verse_box.append(controller.widget());
            verses_controllers.push(controller)
        }

        self.verses = verses_controllers;

        let widgets = view_output!();

        widgets
    }

    fn update(&mut self, msg: Self::Input, sender: FactorySender<Self>) {
        match msg {
            SectionInput::Lookup(text) => {
                let _ = sender.output(SectionOutput::Lookup(text));
            }
            SectionInput::ToggleDisplay(verse_message) => {
                for controller in &self.verses {
                    controller.emit(verse_message.clone());
                }
                self.config.write().unwrap().apply_message(&verse_message);

                for controller in &self.title_word_controllers {
                    controller.emit(WordModelInput::UpdateConfig(self.config.clone()));
                }
            }
        }
    }
}
