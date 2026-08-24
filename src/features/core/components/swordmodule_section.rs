use adw::prelude::*;
use relm4::prelude::*;
use std::collections::HashMap;
use xbible_engine::engines::module_engine::sword_module::module::SwordModule;

use crate::features::core::{
    components::swordmodule::{ModuleTile, ModuleTileInit, ModuleTileInput, ModuleTileOutput}, pages::store::store_page::InstallationStatus,
};

pub struct ModuleSectionInit {
    pub language_name: String,
    pub modules: Vec<SwordModule>,
    pub status_map: HashMap<String, InstallationStatus>,
    pub is_library_mode: bool,
}

pub struct ModuleSection {
    language_name: String,
    is_revealed: bool,
    modules: FactoryVecDeque<ModuleTile>,
    pub name_to_index: HashMap<String, DynamicIndex>,
}

#[derive(Debug)]
pub enum ModuleSectionInput {
    ToggleReveal,
    ChildOutput(ModuleTileOutput),
    ChildInput(String, ModuleTileInput),
}

#[derive(Debug, Clone)]
pub enum ModuleSectionOutput {
    TileAction(ModuleTileOutput),
}

#[relm4::component(pub)]
impl Component for ModuleSection {
    type Init = ModuleSectionInit;
    type Input = ModuleSectionInput;
    type Output = ModuleSectionOutput;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,
            add_css_class: "module-section-container",

            // SECTION HEADER
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 10,
                set_margin_bottom: 10,
                set_halign: gtk::Align::Start,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    set_cursor_from_name: Some("pointer"),

                    // Content inside the header
                    gtk::Box {
                        set_spacing: 10,
                        set_halign: gtk::Align::Start,

                        gtk::Label {
                            set_label: &model.language_name,
                            add_css_class: "title-3",
                            set_halign: gtk::Align::Start,
                        },

                        gtk::Image {
                            #[watch]
                            set_icon_name: Some(if model.is_revealed { "go-down-symbolic" } else { "go-next-symbolic" }),
                            set_halign: gtk::Align::Start,
                        },
                    },

                    add_controller = gtk::GestureClick {
                        connect_released[sender] => move |_, _, _, _| {
                            sender.input(ModuleSectionInput::ToggleReveal);
                        }
                    }
                },

                gtk::Label {
                    set_label: &format!("{} {}", model.modules.len(), if model.modules.len() == 1 { "Book" } else { "Books" }),
                    add_css_class: "dim-label",
                    set_halign: gtk::Align::Start,
                }
            },

            // REVEALER CONTENT
            gtk::Revealer {
                #[watch]
                set_reveal_child: model.is_revealed,
                set_transition_type: gtk::RevealerTransitionType::SlideDown,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_margin_horizontal: 10,
                    set_margin_bottom: 10,

                    // Bind the factory widget directly as local_ref
                    #[local_ref]
                    module_flowbox -> gtk::FlowBox {
                        set_selection_mode: gtk::SelectionMode::None,
                        set_column_spacing: 10,
                        set_row_spacing: 10,
                        set_valign: gtk::Align::Start,
                        set_halign: gtk::Align::Start,
                        add_css_class: "module-grid-canvas",
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        // Pass gtk::FlowBox::default() so Relm4 initializes the factory's root container as a FlowBox
        let mut modules = FactoryVecDeque::builder()
            .launch(gtk::FlowBox::default())
            .forward(sender.input_sender(), ModuleSectionInput::ChildOutput);

        let mut name_to_index = std::collections::HashMap::new();

        {
            let mut guard = modules.guard();
            for module in init.modules {
                let module_name = module.name.clone();
                let status = init
                    .status_map
                    .get(&module_name)
                    .cloned()
                    .unwrap_or(InstallationStatus::Idle);

                let index = guard.push_back(ModuleTileInit {
                    module,
                    status,
                    is_library_mode: init.is_library_mode,
                });

                name_to_index.insert(module_name, index);
            }
        }

        let model = ModuleSection {
            language_name: init.language_name,
            is_revealed: true,
            modules,
            name_to_index,
        };

        // Grab the actual FlowBox widget managed by FactoryVecDeque
        let module_flowbox = model.modules.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            ModuleSectionInput::ToggleReveal => {
                self.is_revealed = !self.is_revealed;
            }
            ModuleSectionInput::ChildOutput(output) => {
                let _ = sender.output(ModuleSectionOutput::TileAction(output));
            }
            ModuleSectionInput::ChildInput(module_name, tile_input) => {
                // O(1) lookup: Skip entirely if this section doesn't own the module
                if let Some(index) = self.name_to_index.get(&module_name) {
                    self.modules.send(index.current_index(), tile_input);
                }
            }
        }
    }
}
