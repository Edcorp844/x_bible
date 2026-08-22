use adw::prelude::*;
use relm4::{Component, ComponentController, ComponentParts, Controller, prelude::*};
use std::collections::BTreeMap;
use std::sync::Arc;
use xbible_engine::engines::{
    module_engine::sword_module::module::SwordModule, xbible_engine::engine::XBibleEngine,
};

use crate::features::core::components::swordmodule_section::{ModuleSection, ModuleSectionInit};



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LibraryPageCategory {
    Bible,
    Commentary,
    Dictionary,
    AudioBible,
    Map,
    Book,
}

impl LibraryPageCategory {
    pub fn from_label(label: &str) -> Self {
        match label {
            "Bible Versions" => Self::Bible,
            "Commentaries" => Self::Commentary,
            "Dictionaries" => Self::Dictionary,
            "Audio Bibles" => Self::AudioBible,
            "Maps" => Self::Map,
            "General Books" => Self::Book,
            _ => Self::Bible,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Bible => format!("Bible Versions"),
            Self::Commentary => format!("Commentaries"),
            Self::Dictionary => format!("Dictionaries"),
            Self::AudioBible => format!("Audio Bibles"),
            Self::Map => format!("Maps"),
            Self::Book => format!("General Books"),
        }
    }
}

pub struct LibraryPage {
    category: LibraryPageCategory,
    engine: Arc<XBibleEngine>,
    // Store Controllers instead of a Factory
    section_controllers: Vec<Controller<ModuleSection>>,
    is_sidebar_visible: bool,
}

#[derive(Debug)]
pub enum LibraryPageInput {
    ModuleSelected(String),
}

#[derive(Debug)]
pub enum LibraryPageOutput {
    ToggleSidebar,
}

#[relm4::component(pub)]
impl Component for LibraryPage {
    type Init = (LibraryPageCategory, Arc<XBibleEngine>, bool);
    type Input = LibraryPageInput;
    type Output = LibraryPageOutput;
    type CommandOutput = ();

    view! {
        adw::NavigationPage {
            #[watch]
            set_title: &model.category.to_string(),

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle { set_title: &model.category.to_string() },

                    pack_start = &gtk::ToggleButton {
                        set_icon_name: "sidebar-show-symbolic",
                        #[watch]
                        set_active: model.is_sidebar_visible,
                        connect_clicked[sender] => move |_| {
                            let _ = sender.output(LibraryPageOutput::ToggleSidebar);
                        }
                    }
                },

                #[wrap(Some)]
                set_content = &gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_vexpand: true,

                    #[name = "section_container"]
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 16,
                        set_margin_all: 24,
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
        let (category, engine, is_sidebar_visible) = init;

        let mut model = LibraryPage {
            category,
            engine,
            section_controllers: Vec::new(),
            is_sidebar_visible,
        };

        let widgets = view_output!();

        // Initial sync
        model.sync_sections(&widgets.section_container, sender);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            LibraryPageInput::ModuleSelected(code) => {
                println!("Module selected: {}", code);
            }
        }
    }
}

impl LibraryPage {
    fn sync_sections(&mut self, container: &gtk::Box, sender: ComponentSender<Self>) {
        // 1. Clear existing controllers and UI widgets
        self.section_controllers.clear();
        while let Some(child) = container.first_child() {
            container.remove(&child);
        }

        // 2. Fetch raw modules based on category
        let raw_modules = match self.category {
            LibraryPageCategory::Bible => self.engine.get_bible_modules(),
            LibraryPageCategory::Commentary => self.engine.get_commentary_modules(),
            LibraryPageCategory::Dictionary => self.engine.get_dictionary_modules(),
            LibraryPageCategory::Book => self.engine.get_book_modules(),
            LibraryPageCategory::Map => self.engine.get_map_modules(),
            LibraryPageCategory::AudioBible => Vec::new(),
        };

        // 3. Group modules by language (using raw code or your isolang helper)
        let mut grouped: BTreeMap<String, Vec<SwordModule>> = BTreeMap::new();
        for module in raw_modules {
            grouped
                .entry(module.language.clone())
                .or_default()
                .push(module);
        }

        // 4. Create and mount new Controllers
        for (lang, modules) in grouped {
            let section_controller = ModuleSection::builder()
                .launch(ModuleSectionInit {
                    language_name: lang,
                    modules, // Ensure this is Vec<SwordModule>
                })
                // The 'msg' here is the String coming from ModuleSection::Output
                .forward(sender.input_sender(), |msg: String| {
                    LibraryPageInput::ModuleSelected(msg)
                });

            container.append(section_controller.widget());
            self.section_controllers.push(section_controller);
        }
    }
}
