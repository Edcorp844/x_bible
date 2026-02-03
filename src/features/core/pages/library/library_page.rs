use adw::prelude::*;
use relm4::{Component, ComponentParts, factory::FactoryVecDeque, prelude::*};
use std::sync::Arc;

use crate::features::core::module_engine::sword_engine::SwordEngine;

// --- 1. The Book Cover Factory Component ---

#[derive(Debug)]
pub struct ModuleItem {
    pub name: String,
    pub description: String,
    pub language: String,
}

#[relm4::factory(pub)]
impl FactoryComponent for ModuleItem {
    type Init = (String, String, String);
    type Input = ();
    type Output = String;
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 12,
            set_margin_horizontal: 6,
            set_margin_vertical: 12,
            set_width_request: 200,
            set_halign: gtk::Align::Center,

            // The Physical Book Shape
            gtk::Box {
                add_css_class: "book-cover",
                set_size_request: (200, 260),
                set_halign: gtk::Align::Center,
                set_overflow: gtk::Overflow::Hidden,

                gtk::Box {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_valign: gtk::Align::Center,
                    set_halign: gtk::Align::Center,
                    set_margin_all: 15,
                    set_orientation: gtk::Orientation::Vertical,

                    gtk::Label {
                        set_label: &self.description,
                        set_wrap: true,
                        set_justify: gtk::Justification::Center,
                        set_max_width_chars: 18,
                        add_css_class: "book-description-text",
                    },

                     gtk::Label {
                        set_label: &self.name,
                        set_wrap: true,
                        set_justify: gtk::Justification::Center,
                        set_max_width_chars: 18,
                        add_css_class: "book-description-text",
                        set_margin_top: 10,
                    }
                }
            },

            gtk::Label {
                set_label: &self.language,
                set_wrap: true,
                set_justify: gtk::Justification::Center,
                set_halign: gtk::Align::Center,
                // 3. Ensure the caption also respects the 200px limit
                set_max_width_chars: 20,
                add_css_class: "book-title-caption",
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            name: init.0,
            description: init.1,
            language: init.2,
        }
    }
}

// --- 2. The Library Page Category Enum ---

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
}

// --- 3. The Main Component ---

pub struct LibraryPage {
    category: LibraryPageCategory,
    engine: Arc<SwordEngine>,
    modules: FactoryVecDeque<ModuleItem>,
    is_sidebar_visible: bool,
}

#[derive(Debug)]
pub enum LibraryPageInput {
    SetCategory(LibraryPageCategory),
    Refresh,
}

#[derive(Debug)]
pub enum LibraryPageOutput {
    ToggleSidebar,
}

#[relm4::component(pub)]
impl Component for LibraryPage {
    type Init = (LibraryPageCategory, Arc<SwordEngine>, bool);
    type Input = LibraryPageInput;
    type Output = LibraryPageOutput;
    type CommandOutput = ();

    view! {
        adw::NavigationPage {
            #[watch]
            set_title: &format!("{:?}", model.category),

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                    add_top_bar = &adw::HeaderBar {
                        #[wrap(Some)]
                        set_title_widget = &adw::WindowTitle {
                            set_title: "XBible",
                        },

                        pack_start = &gtk::ToggleButton {
                            set_icon_name: "sidebar-show-symbolic",
                            // Keep the button toggle state in sync with the actual visibility
                            #[watch]
                            set_active: model.is_sidebar_visible,

                            connect_clicked[sender] => move |_| {
                                sender.output(LibraryPageOutput::ToggleSidebar);
                            }
                        }
                    },

                     #[wrap(Some)]
                    set_content=&gtk::ScrolledWindow {
                        set_hscrollbar_policy: gtk::PolicyType::Never,
                        set_vexpand: true,

                        // We name this container so we can mount the factory into it manually
                        #[name = "library_grid"]
                        gtk::FlowBox {
                            set_valign: gtk::Align::Start,
                            set_max_children_per_line: 8,
                            set_min_children_per_line: 2,
                            set_selection_mode: gtk::SelectionMode::None,
                            set_activate_on_single_click: true,
                            set_margin_all: 24,
                            set_column_spacing: 12,
                            set_row_spacing: 12,
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

        // Use the builder to launch the factory correctly
        let modules = FactoryVecDeque::builder()
            .launch(gtk::FlowBox::default())
            .forward(sender.input_sender(), |_| LibraryPageInput::Refresh);

        let mut model = LibraryPage {
            category,
            engine,
            modules,
            is_sidebar_visible
        };

        // Populate initial data
        model.sync_modules();

        let widgets = view_output!();

        // MANUALLY mount the factory's internal widget into the FlowBox in our view
        widgets.library_grid.append(model.modules.widget());

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            LibraryPageInput::SetCategory(new_cat) => {
                self.category = new_cat;
                self.sync_modules();
            }
            LibraryPageInput::Refresh => {
                self.sync_modules();
            }
        }
    }
}

impl LibraryPage {
    fn sync_modules(&mut self) {
        let mut guard = self.modules.guard();
        guard.clear();

        let modules = match self.category {
            LibraryPageCategory::Bible => self.engine.get_bible_modules(),
            LibraryPageCategory::Commentary => self.engine.get_commentary_modules(),
            LibraryPageCategory::Dictionary => self.engine.get_dictionary_modules(),
            LibraryPageCategory::Book => self.engine.get_book_modules(),
            LibraryPageCategory::Map => self.engine.get_map_modules(),
            // For categories not yet specifically handled in the engine,
            // you can return an empty vec or a general fetcher
            LibraryPageCategory::AudioBible => Vec::new(),
        };

        // 3. Push the results into the UI Factory
        for module in modules {
            guard.push_back((module.name, module.description, module.language));
        }
    }
}
