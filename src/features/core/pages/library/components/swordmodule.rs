use adw::prelude::*;
use relm4::prelude::*;

use crate::{
    features::core::module_engine::sword_module::SwordModule,
    utils::colors_generation::ColorGenerator,
};

/// UI wrapper for the external SwordModule to track local state like hover.
pub struct ModuleTile {
    pub module: SwordModule,
    pub is_hovered: bool,
}

#[relm4::factory(pub)]
impl FactoryComponent for ModuleTile {
    type Init = SwordModule;
    type Input = bool;
    type Output = String;
    type CommandOutput = ();
    type ParentWidget = gtk::FlowBox;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,
            set_margin_all: 12,
            set_width_request: 200,
            set_valign: gtk::Align::Start,

            // Detect Mouse Hover
            add_controller = gtk::EventControllerMotion {
                connect_enter[sender] => move |_, _, _| {
                    sender.input(true);
                },
                connect_leave[sender] => move |_| {
                    sender.input(false);
                }
            },

            // --- THE PHYSICAL BOOK ---
            gtk::Box {
                set_size_request: (200, 260),
                set_halign: gtk::Align::Center,
                set_overflow: gtk::Overflow::Hidden,

                #[watch]
                inline_css: &{
                    let lang_name = &self.module.language;
                    let base_color = ColorGenerator::generate_book_cover_color_according_to_language(&lang_name);

                     format!(
                        "background: linear-gradient(to right, \
                            rgba(0, 0, 0, 0.3) 0%, \
                            rgba(255, 255, 255, 0.1) 5%, \
                            transparent 10%), {}; \
                        border-radius: 4px 12px 12px 4px; \
                        box-shadow: 5px 10px 20px rgba(0, 0, 0, 0.4); \
                        border-left: 3px solid rgba(255, 255, 255, 0.2);",
                        base_color
                    )
                },

                gtk::Box {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_valign: gtk::Align::Center,
                    set_halign: gtk::Align::Center,
                    set_margin_all: 24,
                    set_orientation: gtk::Orientation::Vertical,

                    gtk::Label {
                        set_label: &self.module.description,
                        set_wrap: true,
                        set_justify: gtk::Justification::Center,
                        set_max_width_chars: 14,
                        inline_css: "color: white; font-weight: 700; font-size: 1.1rem; text-shadow: 0 2px 4px rgba(0,0,0,0.5);",
                    },

                    gtk::Label {
                        set_label: &format!("Version {}", self.module.version),
                        set_margin_top: 12,
                        inline_css: "color: rgba(255, 255, 255, 0.7); font-weight: 400; font-size: 0.85rem;",
                    }
                }
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,
                set_margin_top: 4,

                #[watch]
                set_opacity: if self.is_hovered { 1.0 } else { 0.0 },
                inline_css: "transition: opacity 0.25s cubic-bezier(0.4, 0, 0.2, 1);",

                gtk::Label {
                    set_label: &self.module.name,
                    set_hexpand: true,
                    set_halign: gtk::Align::Start,
                    set_ellipsize: gtk::pango::EllipsizeMode::End,
                    set_max_width_chars: 18,
                    inline_css: "font-weight: 600; font-size: 0.95rem; color: @window_fg_color;",
                },

                gtk::Button {
                    set_icon_name: "view-more-horizontal-symbolic",
                    add_css_class: "circular",
                    add_css_class: "flat",
                    set_valign: gtk::Align::Center,
                }
            }
        }
    }

    fn update(&mut self, is_hovered: Self::Input, _sender: FactorySender<Self>) {
        self.is_hovered = is_hovered;
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            module: init,
            is_hovered: false,
        }
    }
}
