use adw::prelude::*;
use std::sync::{Arc, RwLock};
use xbible_engine::engines::{
    module_engine::{module_engine_extensions::module_engine_dictionary_ext::DictionaryQuery, sword_module::{module::SwordModule, module_book::ModuleBook}}, xbible_engine::engine::XBibleEngine,
};

use relm4::{Component, ComponentParts, ComponentSender, Controller, prelude::*};

use crate::features::{
    bible::components::page::{
        biblepage_model::{BiblePage, StudyInput, StudyPageOutput},
        helpers::PageDisplayConfig,
    },
    core::display_configurations::config::TextConfig,
    dictionary::components::dictionary_model::{DictionaryInputMessage, DictionaryPage},
};

pub struct BiblePageRoot {
    bible_page: Controller<BiblePage>,
    dictionary_page: Controller<DictionaryPage>,
    config: TextConfig,
}

#[derive(Debug)]
pub enum BiblePageRootInput {
    HeaderStateChanged(HeaderState),
    GoToReference(String),
    LookupSelectedWord(DictionaryQuery),
    UpdateTheme,
}
#[derive(Debug, Clone)]
pub struct HeaderState {
    pub module: Option<SwordModule>,
    pub book: Option<ModuleBook>,
    pub chapter: Option<i32>,
}

#[derive(Debug)]
pub enum BiblePageRootOutput {
    UpdateTheme,
    ReferenceChanged(String),
    HeaderStateChanged(HeaderState), // Add this variant
}

#[relm4::component(pub)]
impl Component for BiblePageRoot {
    type Init = Arc<XBibleEngine>;
    type Input = BiblePageRootInput;
    type Output = BiblePageRootOutput;
    type CommandOutput = ();

    view! {
        #[root]
        #[name="study_tools_widgets"]
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
                        &gtk::Label::new(Some("Lexicons")),
                        Some("Lex"),
                        "Lexicons"
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
          info!("reached biblepage root init");

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
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            BiblePageRootInput::LookupSelectedWord(query) => {
                self.dictionary_page
                    .emit(DictionaryInputMessage::Lookup(query));
            }
            BiblePageRootInput::UpdateTheme => {
                let theme = self.config.read().unwrap().theme();

                //remove old theme classes first
                let themes = ["Classic", "Modern", "Default", "Compact"];
                for t in themes {
                    let class = format!("preview-area-{}", t);

                    if widgets.study_tools_widgets.has_css_class(&class) {
                        widgets.study_tools_widgets.remove_css_class(&class);
                    }
                }

                widgets
                    .study_tools_widgets
                    .add_css_class(&format!("preview-area-{}", theme));

                let _ = sender.output(BiblePageRootOutput::UpdateTheme);
            }
            BiblePageRootInput::GoToReference(refrence) => {
                self.bible_page.emit(StudyInput::LoadReference(refrence));
            }
            BiblePageRootInput::HeaderStateChanged(header_state) => {
                 self.bible_page.emit(StudyInput::HeaderStateChanged(header_state.clone()));
            },
        }
    }
}
