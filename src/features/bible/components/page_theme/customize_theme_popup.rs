use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use adw::prelude::*;
use relm4::{Component, ComponentParts, prelude::FactoryVecDeque, prelude::*};
use xbible_engine::engines::xbible_engine::engine::XBibleEngine;

use crate::features::{
    bible::components::{
        page::{
            biblepage_settings::BiblePageSettings, helpers::AvailableFonts, section::{SectionInput, SectionModel}, verse_components::verse::VerseInputMessage
        },
        page_theme::theme_data::ThemePreset,
    },
    core::display_configurations::{
            Config::TextConfig, preview_display_configuration::PreviewDisplayConfig,
        },
};

pub struct CustomizeThemePopup {
    engine: Arc<XBibleEngine>,
    config: TextConfig,
    preview: FactoryVecDeque<SectionModel>,
}

#[derive(Debug)]
pub enum CustomizeThemeInput {
    SelectTheme(i32),
    SetPreview(VerseInputMessage),
    SaveConfig,
    ResetTheme,
}

#[derive(Debug)]
pub enum CustomizeThemeOutput {
    SaveConfig(TextConfig),
    Close,
}

#[relm4::component(pub)]
impl Component for CustomizeThemePopup {
    type Init = (TextConfig, Arc<XBibleEngine>);
    type Input = CustomizeThemeInput;
    type Output = CustomizeThemeOutput;
    type CommandOutput = ();

    view! {
        adw::Window {
            set_title: Some("Customize Theme"),
            set_default_width: 700,
            set_default_height: 700,
            set_modal: true,

            adw::ToolbarView {
                // ───────────── Header Bar ─────────────
                add_top_bar = &adw::HeaderBar {
                    #[watch]
                    set_css_classes: &[&model.make_css_preview_clss(model.config.read().unwrap().theme())],
                    set_show_start_title_buttons: false,
                    set_show_end_title_buttons: false,

                    pack_start = &gtk::Button {
                        set_label: "Cancel",
                        connect_clicked[sender] => move |_| {
                            let _ = sender.output(CustomizeThemeOutput::Close);
                        },
                    },

                    pack_end = &gtk::Button {
                        set_label: "Done",
                        add_css_class: "suggested-action",
                        add_css_class: "accent",
                        connect_clicked[sender] => move |_| {
                             sender.input(CustomizeThemeInput::SaveConfig);
                        },
                    },
                },

                // ───────────── Content ─────────────
                #[wrap(Some)]
                set_content = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    gtk::Box{
                        #[watch]
                        set_css_classes: &[&model.make_css_preview_clss(model.config.read().unwrap().theme())],

                        gtk::Label {
                            set_use_markup: true,
                            set_xalign: 0.0,
                            set_margin_all: 20,
                            #[watch]
                            set_markup: &model.font_markup(),
                        },

                        gtk::Overlay {
                            set_margin_horizontal: 20,
                            set_height_request: 200,
                            set_vexpand: false,
                            set_overflow: gtk::Overflow::Hidden,

                            #[wrap(Some)]
                            set_child = &gtk::ScrolledWindow {
                                set_propagate_natural_height: false,
                                set_margin_top: 20,
                                set_min_content_height: 200,
                                set_max_content_height: 200,
                                set_vscrollbar_policy: gtk::PolicyType::External,
                                set_hscrollbar_policy: gtk::PolicyType::Never,

                                #[wrap(Some)]
                                set_child = &gtk::Viewport {
                                    set_scroll_to_focus: false,
                                    set_hexpand: true,

                                    #[wrap(Some)]
                                    #[local_ref]
                                    set_child =  preview_widget -> gtk::Box {
                                        set_orientation: gtk::Orientation::Horizontal,
                                        set_spacing: 5,
                                        set_valign: gtk::Align::Start,
                                        set_vexpand: true,
                                    }
                                }
                            },

                            // The Fading Effect Overlay
                            add_overlay = &gtk::Box {
                                set_valign: gtk::Align::End,
                                set_height_request: 90,
                                set_can_target: false,
                                add_css_class: "preview-fade-overlay",
                            },
                        }
                    },

                    // 3. SCROLLABLE SETTINGS AREA
                    gtk::ScrolledWindow {
                        set_vexpand: true,
                        set_hscrollbar_policy: gtk::PolicyType::Never,

                        adw::Clamp {
                            set_margin_vertical: 20,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 20,

                                adw::PreferencesGroup {
                                    set_title: "Themes",

                                    gtk::FlowBox{
                                        gtk::Box{
                                            #[watch]
                                            set_class_active: ("menu-theme-btn-active", model.config.read().unwrap().theme() == ThemePreset::Default.to_string()),
                                            add_css_class: "menu-theme-btn",
                                            set_halign: gtk::Align::Fill,
                                            set_valign: gtk::Align::Fill,

                                            gtk::Label{
                                                set_label: "Default",
                                                set_halign: gtk::Align::Center,
                                                set_valign: gtk::Align::Center,
                                                set_hexpand: true,
                                                set_vexpand: true,
                                            },

                                             add_controller = gtk::GestureClick {
                                                connect_released[sender] => move |_, _, _, _| {
                                                    let theme = ThemePreset::Default;
                                                    let index = ThemePreset::all().iter()
                                                        .position(|f| *f == theme)
                                                        .unwrap_or(0) as i32;

                                                    sender.input(CustomizeThemeInput::SelectTheme(index));
                                                }
                                            }
                                        },

                                        gtk::Box{
                                            #[watch]
                                            set_class_active: ("menu-theme-btn-active", model.config.read().unwrap().theme() == ThemePreset::Compact.to_string()),
                                            add_css_class: "menu-theme-btn",
                                            add_css_class: "preview-area-Compact",
                                            set_halign: gtk::Align::Fill,
                                            set_valign: gtk::Align::Fill,

                                            gtk::Label{
                                                set_label: "Compact",
                                                set_halign: gtk::Align::Center,
                                                set_valign: gtk::Align::Center,
                                                set_use_markup: true,
                                                set_markup: model.make_markup_for_theme_btn(ThemePreset::Compact).as_str(),
                                                set_hexpand: true,
                                                set_vexpand: true,
                                            },

                                             add_controller = gtk::GestureClick {
                                                connect_released[sender] => move |_, _, _, _| {
                                                    let theme = ThemePreset::Compact;
                                                    let index = ThemePreset::all().iter()
                                                        .position(|f| *f == theme)
                                                        .unwrap_or(0) as i32;

                                                    sender.input(CustomizeThemeInput::SelectTheme(index));
                                                }
                                            }
                                        },

                                        gtk::Box{
                                            #[watch]
                                            set_class_active: ("menu-theme-btn-active", model.config.read().unwrap().theme() == ThemePreset::Classic.to_string()),
                                            add_css_class: "menu-theme-btn",
                                            add_css_class: "preview-area-Classic",
                                            set_halign: gtk::Align::Fill,
                                            set_valign: gtk::Align::Fill,

                                            gtk::Label{
                                                set_label: "Classic",
                                                set_halign: gtk::Align::Center,
                                                set_valign: gtk::Align::Center,
                                                set_hexpand: true,
                                                set_vexpand: true,
                                                set_use_markup: true,
                                                set_markup: model.make_markup_for_theme_btn(ThemePreset::Classic).as_str(),
                                            },

                                             add_controller = gtk::GestureClick {
                                                connect_released[sender] => move |_, _, _, _| {
                                                    let theme = ThemePreset::Classic;
                                                    let index = ThemePreset::all().iter()
                                                        .position(|f| *f == theme)
                                                        .unwrap_or(0) as i32;

                                                    sender.input(CustomizeThemeInput::SelectTheme(index));
                                                }
                                            }
                                        },

                                        gtk::Box{
                                            #[watch]
                                            set_class_active: ("menu-theme-btn-active", model.config.read().unwrap().theme() == ThemePreset::Modern.to_string()),
                                            add_css_class: "menu-theme-btn",
                                            add_css_class: "preview-area-Modern",
                                            set_halign: gtk::Align::Fill,
                                            set_valign: gtk::Align::Fill,

                                            gtk::Label{
                                                set_label: "Modern",
                                                set_halign: gtk::Align::Center,
                                                set_valign: gtk::Align::Center,
                                                set_hexpand: true,
                                                set_vexpand: true,
                                                set_use_markup: true,
                                                set_markup: model.make_markup_for_theme_btn(ThemePreset::Modern).as_str(),
                                            },

                                            add_controller = gtk::GestureClick {
                                                connect_released[sender] => move |_, _, _, _| {
                                                    let theme = ThemePreset::Modern;
                                                    let index = ThemePreset::all().iter()
                                                        .position(|f| *f == theme)
                                                        .unwrap_or(0) as i32;

                                                    sender.input(CustomizeThemeInput::SelectTheme(index));
                                                }
                                            }
                                        },
                                    }
                                },

                                // ───── Text Settings ─────
                                adw::PreferencesGroup {
                                    set_title: "Text",

                                    adw::ComboRow {
                                        set_title: "Font Family",
                                        set_model: Some(&gtk::StringList::new(
                                            &AvailableFonts::all().iter().map(|f| f.to_string()).collect::<Vec<_>>()
                                                .iter().map(|s| s.as_str()).collect::<Vec<_>>()
                                        )),

                                        #[watch]
                                        set_selected: {
                                            let current_font = model.config.read().unwrap().font();
                                            AvailableFonts::all()
                                                .iter()
                                                .position(|f| *f == current_font)
                                                .unwrap_or(0) as u32
                                        },

                                        connect_selected_item_notify[sender] => move |row| {
                                            if let Some(font) = AvailableFonts::all().get(row.selected() as usize) {
                                                sender.input(CustomizeThemeInput::SetPreview(
                                                    VerseInputMessage::ChangeFont(font.clone())
                                                ));
                                            }
                                        }
                                    },

                                    adw::SwitchRow{
                                        set_title: "Bold Text",
                                        #[watch]
                                        set_active: model.config.read().unwrap().bold_font(),
                                        connect_active_notify[sender]=>move |row|{
                                            sender.input(
                                                CustomizeThemeInput::SetPreview(
                                                    VerseInputMessage::ChangeBoldFont(
                                                        row.is_active()
                                                    )
                                                )
                                            );
                                        }
                                    }
                                },

                                // ───── Layout Settings ─────
                                adw::PreferencesGroup {
                                    set_title: "Layout",

                                    adw::PreferencesRow {
                                       #[wrap(Some)]
                                       set_child=&gtk::Box{
                                            set_orientation: gtk::Orientation::Vertical,
                                            gtk::Label{
                                                set_label: "Line Spacing",
                                                set_xalign: 0.0,
                                                set_margin_horizontal: 10,
                                                set_margin_vertical: 10,
                                            },

                                            gtk::Box{
                                                set_orientation: gtk::Orientation::Horizontal,
                                                set_margin_horizontal: 20,
                                                set_margin_vertical: 10,

                                                gtk::Image {
                                                    set_icon_name: Some("line-height-symbolic"),
                                                    set_pixel_size: 24,
                                                },

                                                gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.8, 2.5, 0.01) {
                                                    set_hexpand: true,
                                                    add_css_class: "accent",
                                                    #[watch]
                                                    set_value: model.config.read().unwrap().line_spacing(),

                                                    connect_value_changed[sender] => move |scale| {
                                                       sender.input(
                                                            CustomizeThemeInput::SetPreview(
                                                                VerseInputMessage::ChangeLineSpacing(
                                                                    scale.value()
                                                                )
                                                            )
                                                        )
                                                    }
                                                },

                                                gtk::Label{
                                                    #[watch]
                                                    set_label: format!("{}", model.config.read().unwrap().line_spacing().floor()).as_str()
                                                }
                                            }
                                        }

                                    },

                                    adw::PreferencesRow {
                                       #[wrap(Some)]
                                       set_child=&gtk::Box{
                                            set_orientation: gtk::Orientation::Vertical,
                                            gtk::Label{
                                                set_label: "Word Spacing",

                                                set_xalign: 0.0,
                                                set_margin_horizontal: 10,
                                                set_margin_vertical: 10,
                                            },

                                            gtk::Box{
                                                set_orientation: gtk::Orientation::Horizontal,
                                                set_margin_horizontal: 20,
                                                set_margin_vertical: 10,

                                                gtk::Image {
                                                    set_icon_name: Some("font-letter-symbolic"),
                                                },

                                                gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 20.0, 1.0) {
                                                    set_hexpand: true,
                                                    add_css_class: "accent",
                                                    #[watch]
                                                    set_value: model.config.read().unwrap().word_spacing(),

                                                    connect_value_changed[sender] => move |scale| {
                                                       sender.input(
                                                            CustomizeThemeInput::SetPreview(
                                                                VerseInputMessage::ChangeWordSpacing(
                                                                    scale.value()
                                                                )
                                                            )
                                                        )
                                                    }
                                                },


                                                gtk::Label{
                                                    #[watch]
                                                    set_label: format!("{:.0}% ",((model.config.read().unwrap().word_spacing() - 10.0) * 10.0).round() ).as_str()
                                                }
                                            }
                                        }

                                    },

                                },

                                adw::PreferencesGroup {
                                     adw::SwitchRow{
                                        set_title: "Justify",
                                        #[watch]
                                        set_active: model.config.read().unwrap().justify(),
                                        connect_active_notify[sender]=>move |row|{
                                            sender.input(
                                                CustomizeThemeInput::SetPreview(
                                                    VerseInputMessage::ChangeJustify(
                                                        row.is_active()
                                                    )
                                                )
                                            );
                                        }
                                    },

                                },

                                 // ───── Theme Settings ─────
                                adw::PreferencesGroup {
                                    set_title: "Reset",

                                    adw::ButtonRow {
                                        set_title: "Reset Theme",
                                        add_css_class: "destructive-action",

                                        connect_activated[sender]=>move|_|{
                                            sender.input(CustomizeThemeInput::ResetTheme);
                                        }
                                    }
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
        root: Self::Root,
        sender: relm4::ComponentSender<Self>,
    ) -> relm4::ComponentParts<Self> {
        let (config, engine) = init;
        let preview = FactoryVecDeque::builder()
            .launch(gtk::Box::new(gtk::Orientation::Vertical, 0))
            .detach();

        let config = Arc::new(RwLock::new(PreviewDisplayConfig::from_page_config(config)));
        let mut model = CustomizeThemePopup {
            engine,
            config,
            preview,
        };

        model.setup_preview();
        let preview_widget = model.preview.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            CustomizeThemeInput::ResetTheme => {
                if let Some(theme) = ThemePreset::all().get(
                    ThemePreset::all()
                        .iter()
                        .position(|f| {
                            *f == ThemePreset::from_string(self.config.read().unwrap().theme())
                        })
                        .unwrap_or(0),
                ) {
                    self.config = Arc::new(RwLock::new(PreviewDisplayConfig::from_theme(
                        *theme,
                        self.config.clone(),
                    )));
                };
            }
            CustomizeThemeInput::SelectTheme(index) => {
                if let Some(theme) = ThemePreset::all().get(index as usize) {
                    self.config = Arc::new(RwLock::new(PreviewDisplayConfig::from_theme(
                        *theme,
                        self.config.clone(),
                    )));
                };
            }
            CustomizeThemeInput::SetPreview(msg) => {
                self.config.write().unwrap().apply_message(&msg);
                for i in 0..self.preview.len() {
                    self.preview
                        .send(i, SectionInput::ToggleDisplay(msg.clone()));
                }
            }
            CustomizeThemeInput::SaveConfig => {
                let _ = sender.output(CustomizeThemeOutput::SaveConfig(self.config.clone()));
                let _ = sender.output(CustomizeThemeOutput::Close);
            }
        }
    }
}

impl CustomizeThemePopup {
    fn setup_preview(&mut self) {
        
        let saved_state = BiblePageSettings::load();
        let modules = self.engine.get_bible_modules();

        let active_module = if let Some(saved_name) = saved_state.last_module {
            modules.iter().find(|m| m.name == saved_name).cloned()
        } else {
            modules.first().cloned()
        };
        let preview_sections = self.engine.get_single_entry(&active_module.unwrap(), "John 3:16");
        let mut guard = self.preview.guard();
        guard.clear();
        for section in preview_sections {
            guard.push_back((section, self.config.clone(), HashMap::new()));
        }
    }

    fn font_markup(&self) -> String {
        match self.config.read().unwrap().font() {
            AvailableFonts::System => {
                format!("<span size='x-large'>Aa</span>")
            }
            _ => {
                format!(
                    "<span size='x-large' face='{}'>Aa</span>",
                    self.config.read().unwrap().font().to_string(),
                )
            }
        }
    }

    fn make_css_preview_clss(&self, theme: String) -> String {
        format!("preview-area-{}", theme)
    }

    fn make_markup_for_theme_btn(&self, theme: ThemePreset) -> String {
        let settings = theme.get_settings();
        let weight = if settings.bold_font { "bold" } else { "normal" };
        let letter_spacing = 0;

        format!(
            "<span face='{}' weight='{}' letter_spacing='{}'>{}</span>",
            settings.font.to_string(),
            weight,
            letter_spacing,
            theme.to_string()
        )
    }
}
