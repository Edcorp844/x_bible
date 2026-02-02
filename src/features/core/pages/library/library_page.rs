use adw::prelude::*;
use gtk::prelude::*;
use relm4::{Component, ComponentParts, factory::FactoryVecDeque, prelude::*};
use std::sync::Arc;

use crate::features::core::sword_engine::SwordEngine;

// --- 1. The Book Cover Factory Component ---

#[derive(Debug)]
pub struct ModuleItem {
    pub name: String,
    pub description: String,
}

#[relm4::factory(pub)]
impl FactoryComponent for ModuleItem {
    type Init = (String, String);
    type Input = ();
    type Output = String;
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 12,
            set_margin_all: 12,
          // set_width_request: 170,

            // The Physical Book Shape
            gtk::Box {
                add_css_class: "book-cover",
                set_size_request: (200, 260),
                // Center the book itself in the grid cell
                set_halign: gtk::Align::Center,

                // Inner Box to force vertical and horizontal centering
                gtk::Box {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_valign: gtk::Align::Center,
                    set_halign: gtk::Align::Center,
                    set_margin_all: 15, // Keep text away from book edges
                    set_margin_start: 20,

                    gtk::Label {
                        set_label: &self.description,
                        set_wrap: true,
                        set_lines: 10,
                        set_justify: gtk::Justification::Center,
                        // Remove set_ellipsize so all words show
                        // Remove set_max_width_chars to let wrap handle it
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        add_css_class: "book-description-text",
                    }
                }
            },

            // Module Name (Caption below the book)
            gtk::Label {
                set_label: &self.name,
                set_wrap: true,
                set_justify: gtk::Justification::Center,
                set_halign: gtk::Align::Center,
                add_css_class: "book-title-caption",
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            name: init.0,
            description: init.1,
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

    fn to_sword_filter(&self) -> Vec<&str> {
        match self {
            Self::Bible => vec!["Biblical Texts", "Bibles"],
            Self::Commentary => vec!["Commentaries"],
            Self::Dictionary => vec!["Lexicons", "Dictionaries"],
            Self::Book => vec!["Generic Books"],
            _ => vec![],
        }
    }
}

// --- 3. The Main Component ---

pub struct LibraryPage {
    category: LibraryPageCategory,
    engine: Arc<SwordEngine>,
    modules: FactoryVecDeque<ModuleItem>,
}

#[derive(Debug)]
pub enum LibraryPageInput {
    SetCategory(LibraryPageCategory),
    Refresh,
}

#[relm4::component(pub)]
impl Component for LibraryPage {
    type Init = (LibraryPageCategory, Arc<SwordEngine>);
    type Input = LibraryPageInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        adw::NavigationPage {
            #[watch]
            set_title: &format!("{:?}", model.category),

            #[wrap(Some)]
            set_child = &gtk::ScrolledWindow {
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
                    add_css_class:"flat"
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (category, engine) = init;

        // Use the builder to launch the factory correctly
        let modules = FactoryVecDeque::builder()
            .launch(gtk::FlowBox::default())
            .forward(sender.input_sender(), |_| LibraryPageInput::Refresh);

        let mut model = LibraryPage {
            category,
            engine,
            modules,
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
    /// Filters and loads modules from the engine into the UI factory
    fn sync_modules(&mut self) {
        let mut guard = self.modules.guard();
        guard.clear();

        let filter = self.category.to_sword_filter();
        let available_modules = self.engine.get_modules();

        for module in available_modules {
            if filter.is_empty() || filter.contains(&module.category.as_str()) {
                guard.push_back((module.name, module.description));
            }
        }
    }
}
