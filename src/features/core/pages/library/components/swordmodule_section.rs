use crate::features::core::{
    module_engine::sword_module::SwordModule, pages::library::components::swordmodule::ModuleTile,
};
use adw::prelude::*;
use relm4::prelude::*;

pub struct ModuleSectionInit {
    pub language_name: String,
    pub modules: Vec<SwordModule>,
}

pub struct ModuleSection {
    language_name: String,
    is_revealed: bool,
    modules: FactoryVecDeque<ModuleTile>,
}

#[derive(Debug)]
pub enum ModuleSectionInput {
    ToggleReveal,
}

#[relm4::component(pub)]
impl Component for ModuleSection {
    type Init = ModuleSectionInit;
    type Input = ModuleSectionInput;
    type Output = String; // Forwards the selected module name/code
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,
            add_css_class: "module-section-container",

            // SECTION HEADER
            gtk::Box{
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 12,
                set_margin_all: 12,
                set_halign: gtk::Align::Start,

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 12,
                    add_css_class: "apple-list-header",
                    set_cursor_from_name: Some("pointer"),

                    // Content inside the header
                    gtk::Box {
                        set_spacing: 12,
                        set_halign: gtk::Align::Start,


                        gtk::Label {
                            set_label: &model.language_name,
                            inline_css: "font-weight: 600; font-size: 1.1rem;",
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
                        set_label: &format!("{} {}", model.modules.len(), if model.modules.len() == 1 { "Book"} else {"Books"}),
                        add_css_class: "dim-label",
                        inline_css: "font-weight: 500; font-size: 1.1rem;",
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
                    set_margin_horizontal: 12,
                    set_margin_bottom: 20,

                    // The FlowBox where the Factory lives
                    #[local_ref]
                    module_flowbox -> gtk::FlowBox {
                        set_selection_mode: gtk::SelectionMode::None,
                        set_column_spacing: 20,
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
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut modules = FactoryVecDeque::builder()
            .launch(gtk::FlowBox::default())
            .forward(sender.output_sender(), |msg| msg);

        {
            let mut guard = modules.guard();
            for m in init.modules {
                guard.push_back(m);
            }
        }

        let model = ModuleSection {
            language_name: init.language_name,
            is_revealed: true,
            modules,
        };

        let module_flowbox = model.modules.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            ModuleSectionInput::ToggleReveal => {
                self.is_revealed = !self.is_revealed;
            }
        }
    }
}
