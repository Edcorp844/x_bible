use gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, prelude::*};
use std::sync::Arc;

use crate::features::core::module_engine::{
    sword_engine::SwordEngine, sword_engine_dictionary_ext::DictionaryQuery,
};

pub struct DictionaryPage {
    engine: Arc<SwordEngine>,
    key: String,
    definition: String,
    lexicon: String,
}

#[derive(Debug)]
pub enum DictionaryInputMessage {
    Lookup(DictionaryQuery),
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

             gtk::Box{
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: false,
                set_margin_all: 20,
                set_halign: gtk::Align::Start,

                gtk::Label {
                        #[watch]
                        set_label: &model.key,
                        set_wrap: true,
                        set_xalign: 0.0,
                        set_yalign: 0.0,
                        set_selectable: true,
                        set_justify: gtk::Justification::Left,
                        set_use_markup: false,
                         add_css_class: "title-4",
                    },

                gtk::Box{
                    add_css_class: "key-underline",
                    set_margin_top: 5,
                    set_hexpand: false,
                    set_width_request: 50,
                    set_height_request: 4,
                    set_halign: gtk::Align::Start,
                },
            },

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
                    set_use_markup: true,
                    set_margin_all: 20,
                },

                #[name = "lexicon_label"]
                gtk::Label {
                    #[watch]
                    set_label: &model.lexicon,
                    set_wrap: true,
                    set_xalign: 0.0,
                    set_yalign: 0.0,
                    set_selectable: true,
                    set_justify: gtk::Justification::Left,
                    set_use_markup: true,
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
            key: "Heavens".to_string(),
            definition: "".to_string(),
            lexicon: "".to_string(),
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
            DictionaryInputMessage::Lookup(query) => {
                self.key = query.clone().word;
                let lookup_result = self.engine.lookup_dictionary(query);
                println!("Recieved: {:?}", lookup_result);
            }
        }
    }
}
