use adw::prelude::*;
use relm4::prelude::*;
use std::sync::{Arc, RwLock};

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

pub struct StudyPage {
    is_sidebar_visible: bool,
    bible_page: Controller<BiblePage>,
    dictionary_page: Controller<DictionaryPage>,
    config: TextConfig,
}

#[derive(Debug)]
pub enum StudyPageInput {
    LookupSelectedStrong(DictionaryQuery),
    UpdateTheme,
}

#[derive(Debug)]
pub enum StudyPageOutPut {
    ToggleSidebar,
}

#[relm4::component(pub)]
impl Component for StudyPage {
    type Init = (Arc<SwordEngine>, bool);
    type Input = StudyPageInput;
    type Output = StudyPageOutPut;
    type CommandOutput = ();

    view! {
        adw::NavigationPage {
            set_title: "Bible Study",
            #[wrap(Some)]
            set_child = &adw::NavigationView {
                push = &adw::NavigationPage {
                    set_title: "Bible Study",
                    #[wrap(Some)]
                    set_child = &adw::ToolbarView {
                        #[name="header"]
                        add_top_bar = &adw::HeaderBar {
                            #[watch]
                            set_css_classes: &[format!("preview-area-{}", (model.config.read().unwrap().theme())).as_str()],

                            pack_start = &gtk::ToggleButton {
                                set_icon_name: "sidebar-show-symbolic",
                                #[watch]
                                set_active: model.is_sidebar_visible,
                                connect_clicked[sender] => move |_| {
                                    let _ = sender.output(StudyPageOutPut::ToggleSidebar);
                                }
                            }
                        },

                        #[name = "main_split"]
                        #[wrap(Some)]
                        set_content = &gtk::Paned{
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
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (engine, is_sidebar_visible) = init;

        let bible_page = BiblePage::builder().launch(engine.clone()).forward(
            sender.input_sender(),
            move |msg| match msg {
                StudyPageOutput::ChangeTheme => StudyPageInput::UpdateTheme,
                StudyPageOutput::LookupSelectedStrong(query) => {
                    StudyPageInput::LookupSelectedStrong(query)
                }
            },
        );

        let dictionary_page = DictionaryPage::builder().launch(engine.clone()).detach();

        let model = StudyPage {
            is_sidebar_visible,
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
            StudyPageInput::UpdateTheme => {
                let theme = self.config.read().unwrap().theme();

                //remove old theme classes first
                let themes = ["Classic", "Modern", "Default", "Compact"];
                for t in themes {
                    let class = format!("preview-area-{}", t);
                    if widgets.header.has_css_class(&class) {
                        widgets.header.remove_css_class(&class);
                    }
                    if widgets.main_split.has_css_class(&class) {
                        widgets.main_split.remove_css_class(&class);
                    }
                }

                widgets
                    .header
                    .add_css_class(&format!("preview-area-{}", theme));
                widgets
                    .main_split
                    .add_css_class(&format!("preview-area-{}", theme));
            }

            StudyPageInput::LookupSelectedStrong(query) => {
                self.dictionary_page
                    .emit(DictionaryInputMessage::Lookup(query));
            }
        }
    }
}
