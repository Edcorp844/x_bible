use gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, prelude::*};

pub struct InteractiveNavigationCard {
    is_revealed: bool,
}

#[derive(Debug, Clone)]
pub enum InteractiveNavigationCardInput {
    ToggleReveal,
}

#[relm4::component(pub)]
impl Component for InteractiveNavigationCard {
    type Init = ();
    type Input = InteractiveNavigationCardInput;
    type Output = ();
    type CommandOutput = ();

    view! {
            #[name = "root"]
            gtk::Box{
                #[watch]
                 inline_css: &format!("background-color: {}; border-radius: 16px;  box-shadow: 0px 0px 0px rgba(0,0,0,0.4); border: 1px solid rgba(255,255,255,0.1);", if model.is_revealed { "rgba(7, 7, 7, 1)" } else { "rgba(7, 7, 7, 0.59)" }),
                 set_margin_horizontal: 20,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_halign: gtk::Align::Fill,
                inline_css: "background-color: rgba(255, 255, 255, 0.54); border-radius: 16px; padding: 16px; box-shadow: 0px 12px 32px rgba(0,0,0,0.4); border: 1px solid rgba(255,255,255,0.1);",
                set_hexpand: true,

                // Header Container
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 6,
                    set_halign: gtk::Align::Start,

                    gtk::Label {
                        set_label: "Chapter 1",
                        add_css_class: "title-3",
                    },

                    gtk::Label {
                        set_label: "Chapter 1 of 6",
                        set_halign: gtk::Align::Start,
                    },

                    add_controller = gtk::GestureClick {
                        // Safe, non-panicking event bridge
                        connect_released[sender] => move |_, _, _, _| {
                            sender.input(InteractiveNavigationCardInput::ToggleReveal);
                        }
                    }
                },

                // Clean, isolated sibling layout element
                gtk::Revealer {
                    // Relm4 track tracking variable connection syntax
                    #[watch]
                    set_reveal_child: model.is_revealed,
                    set_transition_type: gtk::RevealerTransitionType::SlideDown,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_margin_horizontal: 10,
                        set_margin_bottom: 10,

                        gtk::Label {
                            set_label: "Expanded Content Details Go Here",
                            add_css_class: "body",
                        }
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = InteractiveNavigationCard { is_revealed: false };
        let widgets = view_output!();
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            InteractiveNavigationCardInput::ToggleReveal => {
                self.is_revealed = !self.is_revealed;
            }
        }
    }
}
