use adw::prelude::*;
use relm4::prelude::*;
use xbible_engine::engines::module_engine::sword_module::module::SwordModule;

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
                        let h = self.module.signature_color.hue;
                        let s_b = self.module.signature_color.saturation;
                        let b = self.module.signature_color.brightness;

                        // 1. Convert HSB to HSL
                        let l = b * (1.0 - s_b / 2.0);
                        let s_l = if l == 0.0 || l == 1.0 {
                            0.0
                        } else {
                            (b - l) / l.min(1.0 - l)
                        };

                        // 2. Dampen the vibrancy to match SwiftUI's rendering profile
                        // Scale saturation down by ~15% and slightly compress lightness
                        let final_s = (s_l * 0.85).clamp(0.0, 1.0);
                        let final_l = (l * 0.90).clamp(0.0, 1.0);

                        // 3. Format explicitly for GTK
                        let base_color = format!(
                            "hsl({}, {}%, {}%)", 
                            (h * 360.0).round() as u16, 
                            (final_s * 100.0).round() as u16, 
                            (final_l * 100.0).round() as u16
                        );

                        format!(
                            "background-image: linear-gradient(to right, \
                                rgba(0, 0, 0, 0.40) 0%, \
                                rgba(255, 255, 255, 0.12) 6%, \
                                transparent 15%); \
                            background-color: {}; \
                            border-radius: 4px 12px 12px 4px; \
                            box-shadow: 5px 10px 20px rgba(0, 0, 0, 0.45); \
                            border-left: 3px solid rgba(255, 255, 255, 0.15);",
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
                            set_max_width_chars: 16,
                            inline_css: "color: white; font-weight: 800; font-size: 1.1rem; text-shadow: 0 2px 4px rgba(0,0,0,0.8);",
                        },


                        gtk::Label {
                            set_label: &format!("Version {}", self.module.version),
                            set_wrap: true,
                            set_justify: gtk::Justification::Center,
                            set_max_width_chars: 16,
                            set_margin_top: 15,
                            inline_css: "color: rgba(255, 255, 255, 0.7); font-weight: 400; font-size: 0.9rem;",
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
