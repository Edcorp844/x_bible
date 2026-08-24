use adw::prelude::*;
use gtk::glib;
use relm4::prelude::*;
use xbible_engine::engines::module_engine::sword_module::module::SwordModule;

use crate::features::core::pages::store::store_page::InstallationStatus;

pub struct ModuleTileInit {
    pub module: SwordModule,
    pub status: InstallationStatus,
    pub is_library_mode: bool,
}

pub struct ModuleTile {
    pub module: SwordModule,
    pub status: InstallationStatus,
    pub is_library_mode: bool,
    pub is_hovered: bool,
    popover_menu: gtk::PopoverMenu,
}

#[derive(Debug, Clone)]
pub enum ModuleTileInput {
    SetHovered(bool),
    UpdateStatus(InstallationStatus),
    PrimaryActionClicked,
    MenuOptionClicked(ModuleMenuAction),
}

#[derive(Debug, Clone)]
pub enum ModuleMenuAction {
    OpenInStudy,
    Update,
    Delete,
}

#[derive(Debug, Clone)]
pub enum ModuleTileOutput {
    ActionTriggered(SwordModule),
    MenuActionTriggered {
        module: SwordModule,
        action: ModuleMenuAction,
    },
}

#[relm4::factory(pub)]
impl FactoryComponent for ModuleTile {
    type Init = ModuleTileInit;
    type Input = ModuleTileInput;
    type Output = ModuleTileOutput;
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
                    sender.input(ModuleTileInput::SetHovered(true));
                },
                connect_leave[sender] => move |_| {
                    sender.input(ModuleTileInput::SetHovered(false));
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

                    let l = b * (1.0 - s_b / 2.0);
                    let s_l = if l == 0.0 || l == 1.0 {
                        0.0
                    } else {
                        (b - l) / l.min(1.0 - l)
                    };

                    let final_s = (s_l * 0.85).clamp(0.0, 1.0);
                    let final_l = (l * 0.90).clamp(0.0, 1.0);

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

            // --- BOTTOM INFO & ACTION BAR ---
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 8,
                set_margin_top: 4,

                #[watch]
                set_opacity: if self.is_hovered { 1.0 } else { 0.85 },
                inline_css: "transition: opacity 0.25s cubic-bezier(0.4, 0, 0.2, 1);",

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,
                    set_valign: gtk::Align::Center,

                    gtk::Label {
                        set_label: &self.module.name,
                        set_halign: gtk::Align::Start,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_max_width_chars: 14,
                        inline_css: "font-weight: 700; font-size: 0.9rem; color: @window_fg_color;",
                    },

                    gtk::Label {
                        set_label: &self.module.description,
                        set_halign: gtk::Align::Start,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_max_width_chars: 14,
                        inline_css: "font-weight: 500; font-size: 0.8rem; opacity: 0.7; color: @window_fg_color;",
                    }
                },

                // Context Menu Button (Shows in Library Mode OR when Installed)
                #[watch]
                set_visible: self.is_library_mode || matches!(self.status, InstallationStatus::Installed),
                gtk::MenuButton {
                    set_icon_name: "view-more-symbolic",
                    add_css_class: "circular",
                    add_css_class: "flat",
                    set_valign: gtk::Align::Center,
                    set_popover: Some(&self.popover_menu),
                },

                // Action UI based on installation status (Hidden in Library Mode)
                #[watch]
                set_visible: !self.is_library_mode,
                gtk::Box {
                    set_valign: gtk::Align::Center,

                    match &self.status {
                        InstallationStatus::Idle => {
                            gtk::Button {
                                set_label: "Get",
                                add_css_class: "suggested-action",
                                connect_clicked[sender] => move |_| {
                                    sender.input(ModuleTileInput::PrimaryActionClicked);
                                }
                            }
                        },
                        InstallationStatus::Installed => {
                            gtk::Button {
                                set_label: "Open",
                                add_css_class: "flat",
                                connect_clicked[sender] => move |_| {
                                    sender.input(ModuleTileInput::PrimaryActionClicked);
                                }
                            }
                        },
                        InstallationStatus::Pending => {
                            gtk::Spinner {
                                set_spinning: true,
                                set_size_request: (28, 28),
                            }
                        },
                        InstallationStatus::Installing(progress) => {
                            gtk::Overlay {
                                #[wrap(Some)]
                                set_child = &gtk::Spinner {
                                    set_spinning: true,
                                    set_size_request: (28, 28),
                                },
                                add_overlay = &gtk::Label {
                                    #[watch]
                                    set_label: &format!("{}%", (progress * 100.0).round() as u32),
                                    inline_css: "font-size: 0.65rem; font-weight: 800;",
                                    set_halign: gtk::Align::Center,
                                    set_valign: gtk::Align::Center,
                                }
                            }
                        },
                        InstallationStatus::Failed(_) | InstallationStatus::Cancelled => {
                            gtk::Button {
                                set_label: "Retry",
                                add_css_class: "destructive-action",
                                add_css_class: "pill",
                                connect_clicked[sender] => move |_| {
                                    sender.input(ModuleTileInput::PrimaryActionClicked);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, sender: FactorySender<Self>) -> Self {
        let menu_model = gtk::gio::Menu::new();

        if init.is_library_mode {
            menu_model.append(Some("Open in Study"), Some("tile.open_study"));
        } else {
            menu_model.append(Some("Update"), Some("tile.update"));
        }
        menu_model.append(Some("Delete"), Some("tile.delete"));

        let popover_menu = gtk::PopoverMenu::from_model(Some(&menu_model));

        let action_group = gtk::gio::SimpleActionGroup::new();

        let s_study = sender.clone();
        let action_study = gtk::gio::SimpleAction::new("open_study", None);
        action_study.connect_activate(move |_, _| {
            s_study.input(ModuleTileInput::MenuOptionClicked(ModuleMenuAction::OpenInStudy));
        });
        action_group.add_action(&action_study);

        let s_update = sender.clone();
        let action_update = gtk::gio::SimpleAction::new("update", None);
        action_update.connect_activate(move |_, _| {
            s_update.input(ModuleTileInput::MenuOptionClicked(ModuleMenuAction::Update));
        });
        action_group.add_action(&action_update);

        let s_delete = sender.clone();
        let action_delete = gtk::gio::SimpleAction::new("delete", None);
        action_delete.connect_activate(move |_, _| {
            s_delete.input(ModuleTileInput::MenuOptionClicked(ModuleMenuAction::Delete));
        });
        action_group.add_action(&action_delete);

        popover_menu.insert_action_group("tile", Some(&action_group));

        Self {
            module: init.module,
            status: init.status,
            is_library_mode: init.is_library_mode,
            is_hovered: false,
            popover_menu,
        }
    }

    fn update(&mut self, input: Self::Input, sender: FactorySender<Self>) {
        match input {
            ModuleTileInput::SetHovered(val) => {
                self.is_hovered = val;
            }
            ModuleTileInput::UpdateStatus(status) => {
                self.status = status;
            }
            ModuleTileInput::PrimaryActionClicked => {
                let _ = sender.output(ModuleTileOutput::ActionTriggered(self.module.clone()));
            }
            ModuleTileInput::MenuOptionClicked(action) => {
                let _ = sender.output(ModuleTileOutput::MenuActionTriggered {
                    module: self.module.clone(),
                    action,
                });
            }
        }
    }
}