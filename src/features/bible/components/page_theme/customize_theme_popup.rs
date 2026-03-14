use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use adw::prelude::*;
use relm4::{Component, ComponentParts, prelude::FactoryVecDeque, prelude::*};

use crate::features::{
    bible::components::{
        page::{
            helpers::AvailableFonts,
            section::{SectionInput, SectionModel},
            verse_components::verse::VerseInputMessage,
        },
        page_theme::theme_data::ThemePreset,
    },
    core::{
        display_configurations::{
            Config::TextConfig, preview_display_configuration::PreviewDisplayConfig,
        },
        osis_translation_engine::engine::OsisTransilationEngine,
    },
};

pub struct CustomizeThemePopup {
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
    type Init = TextConfig;
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
        let preview = FactoryVecDeque::builder()
            .launch(gtk::Box::new(gtk::Orientation::Vertical, 0))
            .detach();

        let config = Arc::new(RwLock::new(PreviewDisplayConfig::from_page_config(init)));
        let mut model = CustomizeThemePopup { config, preview };

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
        let preview_osis = r#"<w lemma="strong:G1161 lemma.TR:δε" morph="robinson:CONJ" src="2">And</w> <w lemma="strong:G1492 lemma.TR:οιδαμεν" morph="robinson:V-RAI-1P" src="1">we know</w> <w lemma="strong:G3754 lemma.TR:οτι" morph="robinson:CONJ" src="3">that</w> <w lemma="strong:G3588 strong:G5207 lemma.TR:ο lemma.TR:υιος" morph="robinson:T-NSM robinson:N-NSM" src="4 5">the Son</w> <w lemma="strong:G3588 strong:G2316 lemma.TR:του lemma.TR:θεου" morph="robinson:T-GSM robinson:N-GSM" src="6 7">of God</w> <w lemma="strong:G2240 lemma.TR:ηκει" morph="robinson:V-PAI-3S" src="8">is come</w>, <w lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ" src="9">and</w> <w lemma="strong:G1325 lemma.TR:δεδωκεν" morph="robinson:V-RAI-3S" src="10">hath given</w> <w lemma="strong:G1473 lemma.TR:ημιν" morph="robinson:P-1DP" src="11">us</w> <w lemma="strong:G1271 lemma.TR:διανοιαν" morph="robinson:N-ASF" src="12">an understanding</w>, <w lemma="strong:G2443 lemma.TR:ινα" morph="robinson:CONJ" src="13">that</w> <w lemma="strong:G1097 lemma.TR:γινωσκωμεν" morph="robinson:V-PAS-1P" src="14">we may know</w> <w lemma="strong:G228 lemma.TR:αληθινον" morph="robinson:A-ASM" src="16">him that is true</w>, <w lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ" src="17">and</w> <w lemma="strong:G1510 lemma.TR:εσμεν" morph="robinson:V-PAI-1P" src="18">we are</w> <w lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP" src="19">in</w> <w lemma="strong:G228 lemma.TR:αληθινω" morph="robinson:A-DSM" src="21">him that is true</w>, <transChange type="added">even</transChange> <w lemma="strong:G1722 lemma.TR:εν" morph="robinson:PREP" src="22">in</w> <w lemma="strong:G846 lemma.TR:αυτου" morph="robinson:P-GSM" src="25">his</w> <w lemma="strong:G3588 strong:G5207 lemma.TR:τω lemma.TR:υιω" morph="robinson:T-DSM robinson:N-DSM" src="23 24">Son</w> <w lemma="strong:G2424 lemma.TR:ιησου" morph="robinson:N-DSM" src="26">Jesus</w> <w lemma="strong:G5547 lemma.TR:χριστω" morph="robinson:N-DSM" src="27">Christ</w>. <w lemma="strong:G3778 lemma.TR:ουτος" morph="robinson:D-NSM" src="28">This</w> <w lemma="strong:G1510 lemma.TR:εστιν" morph="robinson:V-PAI-3S" src="29">is</w> <w lemma="strong:G228 lemma.TR:αληθινος" morph="robinson:A-NSM" src="31">the true</w> <w lemma="strong:G2316 lemma.TR:θεος" morph="robinson:N-NSM" src="32">God</w>, <w lemma="strong:G2532 lemma.TR:και" morph="robinson:CONJ" src="33">and</w> <w lemma="strong:G166 lemma.TR:αιωνιος" morph="robinson:A-NSF" src="36">eternal</w> <w lemma="strong:G3588 strong:G2222 lemma.TR:η lemma.TR:ζωη" morph="robinson:T-NSF robinson:N-NSF" src="34 35">life</w>.<w lemma="strong:G3588 lemma.TR:τον" morph="robinson:T-ASM" src="15"/><w lemma="strong:G3588 lemma.TR:τω" morph="robinson:T-DSM" src="20"/><w lemma="strong:G3588 lemma.TR:ο" morph="robinson:T-NSM" src="30"/>"#;
        let osis_engine = OsisTransilationEngine::new();
        let verses = osis_engine.parse_osis_to_sections(preview_osis, None);

        let mut guard = self.preview.guard();
        guard.clear();
        for verse in verses {
            guard.push_back((verse, self.config.clone(), HashMap::new()));
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
