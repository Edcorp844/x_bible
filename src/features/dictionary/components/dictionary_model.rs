use gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, prelude::*};
use std::sync::Arc;

use crate::features::core::module_engine::{
    sword_engine::SwordEngine, sword_engine_dictionary_ext::DictionaryQuery,
};

pub struct DictionaryPage {
    engine: Arc<SwordEngine>,
    key: String,
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
            set_spacing: 0,
            set_vexpand: true,
            set_hexpand: true,

            // Header Section
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_margin_all: 24,
                set_halign: gtk::Align::Start,

                #[name = "header_label"]
                gtk::Label {
                    #[watch]
                    set_label: &model.key,
                    set_wrap: true,
                    set_xalign: 0.0,
                    add_css_class: "title-1", // Large bold header
                },

                gtk::Box {
                    add_css_class: "key-underline",
                    set_margin_top: 8,
                    set_width_request: 60,
                    set_height_request: 4,
                    set_halign: gtk::Align::Start,
                },
            },

            // Scrolled area for definitions
            gtk::ScrolledWindow {
                set_hscrollbar_policy: gtk::PolicyType::Never,
                set_vscrollbar_policy: gtk::PolicyType::Automatic,
                set_vexpand: true,
                set_min_content_height: 300,

                #[name = "results_list"]
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 24,
                    set_margin_horizontal: 24,
                    set_margin_bottom: 40,
                    // Dynamic results (Headings + Definitions) injected here
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
            key: "Dictionary".to_string(),
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
                // Update the visible header key immediately
                let new_key = if !query.word.is_empty() {
                    query.word.clone()
                } else {
                    query
                        .strongs
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "Lookup".to_string())
                };
                self.key = new_key.clone();

                // 2. FORCE the Widget Update (This fixes the "Dictionary" stuck bug)
                widgets.header_label.set_label(&new_key);

                // Perform the backend search
                let lookup_result = self.engine.lookup_dictionary(query);

                // 1. Clear the results box for the new search
                while let Some(child) = widgets.results_list.first_child() {
                    widgets.results_list.remove(&child);
                }

                // 2. Handle empty results
                if lookup_result.results.is_empty() {
                    let empty_label = gtk::Label::builder()
                        .label("No definitions found in installed modules.")
                        .css_classes(vec!["dim-label"])
                        .wrap(true)
                        .xalign(0.0)
                        .margin_top(20)
                        .build();
                    widgets.results_list.append(&empty_label);
                    return;
                }

                // 3. Populate results with Source Heading + Definition Body
                for result in lookup_result.results {
                    let reuslt_body = self.engine.format_for_pango(&result.definition);
                    let result_box = gtk::Box::builder()
                        .orientation(gtk::Orientation::Vertical)
                        .spacing(8)
                        .build();

                    // Module Name Heading (e.g., "Webster 1828")
                    let source_heading = gtk::Label::builder()
                        .label(&result.module_name)
                        .wrap(true)
                        .xalign(0.0)
                        .css_classes(vec!["title-4", "accent"])
                        .build();

                    // Definition Text (Supports Pango Markup)
                    let definition_body = gtk::Label::builder()
                        .label(reuslt_body)
                        .use_markup(true)
                        .wrap(true)
                        .selectable(true)
                        //.links(true)
                        .xalign(0.0)
                        .justify(gtk::Justification::Left)
                        .build();

                    // Add a separator for better visual parsing
                    let separator = gtk::Separator::builder()
                        .orientation(gtk::Orientation::Horizontal)
                        .margin_top(16)
                        .opacity(0.3)
                        .build();

                    result_box.append(&source_heading);
                    result_box.append(&definition_body);
                    result_box.append(&separator);

                    widgets.results_list.append(&result_box);
                }
            }
        }
    }
}
