use adw::prelude::*;
use std::sync::{Arc, RwLock};

use relm4::{Component, ComponentParts, ComponentSender, Controller, prelude::*};

use crate::features::{
    bible::components::page::{
        biblepage_model::{BiblePage, StudyPageOutput},
        helpers::PageDisplayConfig,
    },
    core::{
        display_configurations::Config::TextConfig,
        module_engine::{sword_engine::SwordEngine, sword_engine_dictionary_ext::DictionaryQuery},
    },
    dictionary::components::dictionary_model::{DictionaryInputMessage, DictionaryPage},
};

pub struct BiblePageRoot {
    engine: Arc<SwordEngine>,
    bible_page: Controller<BiblePage>,
    dictionary_page: Controller<DictionaryPage>,
    config: TextConfig,
}

#[derive(Debug)]
pub enum BiblePageRootInput {
    LookupSelectedWord(DictionaryQuery),
    UpdateTheme,
}

#[relm4::component(pub)]
impl Component for BiblePageRoot {
    type Init = (Arc<SwordEngine>);
    type Input = BiblePageRootInput;
    type Output = ();
    type CommandOutput = ();

    view! {
    #[root]
    gtk::Paned{
                        set_orientation: gtk::Orientation::Horizontal,
                        set_wide_handle: true,
                        set_shrink_start_child: true,
                        #[watch]
                        set_css_classes: &[format!("preview-area-{}", (model.config.read().unwrap().theme())).as_str(),],

                        set_start_child=Some(model.bible_page.widget()),

                        #[wrap(Some)]
                        set_end_child = &adw::ToolbarView {

                        set_margin_horizontal: 20,
                        add_top_bar: switcher_bar = &adw::InlineViewSwitcher {
                            #[watch]
                            set_stack: Some(&stack),
                            add_css_class: "round",

                        },

                        #[wrap(Some)]
                        set_content: stack = &adw::ViewStack {
                            set_vexpand: true,

                            add_titled: (
                                model.dictionary_page.widget(),
                                Some("dict"),
                                "Dictionary"
                            ),

                            // Page 2: References
                            add_titled: (
                                &gtk::Label::new(Some("Cross References")),
                                Some("ref"),
                                "References"
                            ),

                            add_titled: (
                                &gtk::Label::new(Some("Commentary")),
                                Some("Comm"),
                                "Commentaries"
                            ),

                        }
                    }

                    }
                }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let engine = init;

        let bible_page = BiblePage::builder().launch(engine.clone()).forward(
            sender.input_sender(),
            move |msg| match msg {
                StudyPageOutput::ChangeTheme => BiblePageRootInput::UpdateTheme,
                StudyPageOutput::LookupSelectedStrong(query) => {
                    BiblePageRootInput::LookupSelectedWord(query)
                }
            },
        );

        let dictionary_page = DictionaryPage::builder().launch(engine.clone()).detach();

        let model = Self {
            engine,
            bible_page: bible_page,
            dictionary_page: dictionary_page,
            config: Arc::new(RwLock::new(PageDisplayConfig::new())),
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
            BiblePageRootInput::LookupSelectedWord(query) => {
                self.dictionary_page
                    .emit(DictionaryInputMessage::Lookup(query));
            }
            _ => {}
        }
    }
}
