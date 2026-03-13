use gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, prelude::*};
use std::sync::Arc;

use crate::features::core::module_engine::sword_engine::SwordEngine;

pub struct DictionaryPage {
    engine: Arc<SwordEngine>,
    definition: String,
}

#[derive(Debug)]
pub enum DictionaryInputMessage {
    Lookup(String),
}

#[relm4::component(pub)]
impl Component for DictionaryPage {
    type Init = Arc<SwordEngine>;
    type Input = DictionaryInputMessage;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[name = "dictionary_container"]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,
            set_vexpand: true,
            set_hexpand: true,

            gtk::ScrolledWindow {
                set_hscrollbar_policy: gtk::PolicyType::Never,
                set_vscrollbar_policy: gtk::PolicyType::Automatic,
                set_vexpand: true,

                // This ensures the scrolled window doesn't collapse
                set_min_content_height: 200,

                #[name = "definition_label"]
                gtk::Label {
                    #[watch]
                    set_label: &model.definition,
                    set_wrap: true,
                    set_xalign: 0.0,
                    set_yalign: 0.0,
                    set_selectable: true,
                    set_justify: gtk::Justification::Left,
                    // Use Pango markup if your SWORD engine returns HTML/styles
                    set_use_markup: false,
                    set_margin_all: 20,
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = DictionaryPage {
            engine: init,
            definition: "Enter a word to look up...".to_string(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            DictionaryInputMessage::Lookup(text) => {
                let key = text.trim();
                if !key.is_empty() {
                    let def = self.engine.lookup_definition(key);

                    // Direct UI update to bypass potential #[watch] delay
                    widgets.definition_label.set_label(&def);
                    self.definition = def;
                }
            }
        }
    }
}
