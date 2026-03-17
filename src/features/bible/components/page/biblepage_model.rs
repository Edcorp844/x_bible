use adw::prelude::*;
use gtk::glib::clone;
use relm4::prelude::*;
use std::sync::{Arc, RwLock};

use crate::features::bible::components::page::helpers::{AddedWordStyle, PageDisplayConfig};
use crate::features::bible::components::page::section::{
    SectionInput, SectionModel, SectionOutput,
};
use crate::features::bible::components::page::verse_components::verse::VerseInputMessage;
use crate::features::bible::components::page::verse_components::verse_annotation::{
    AnnotationSettings, Annotations,
};
use crate::features::bible::components::page_theme::customize_theme_popup::{
    CustomizeThemeOutput, CustomizeThemePopup,
};
use crate::features::core::display_configurations::Config::TextConfig;
use crate::features::core::module_engine::sword_engine::SwordEngine;
use crate::features::core::module_engine::sword_engine_dictionary_ext::DictionaryQuery;
use crate::features::core::module_engine::sword_module::SwordModule;
pub struct BiblePage {
    pub(crate) engine: Arc<SwordEngine>,
    pub(crate) module: SwordModule,
    pub(crate) sections: FactoryVecDeque<SectionModel>,
    pub(crate) config: TextConfig,
    pub(crate) customize_theme_popup: Option<Controller<CustomizeThemePopup>>,
    pub(crate) annotations: Annotations,
}

#[derive(Debug)]
pub enum StudyInput {
    LoadReference(String),
    SetModule(String),
    ToggleDisplay(VerseInputMessage),
    SetConfig(TextConfig),
    OpenCustomizethemePopup,
    CloseCustomizethemePopup,
}

#[derive(Debug)]
pub enum StudyPageOutput {
    ChangeTheme,
    LookupSelectedStrong(DictionaryQuery),
}

#[relm4::component(pub)]
impl Component for BiblePage {
    type Init = (Arc<SwordEngine>, String, String);
    type Input = StudyInput;
    type Output = StudyPageOutput;
    type CommandOutput = ();

    view! {
        adw::NavigationPage {
            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                #[name="page_overlay"]
                gtk::Overlay {
                    set_vexpand: true,
                    #[watch]
                    set_css_classes: &[
                        "page-overlay",
                        &model.make_css_preview_clss(model.config.read().unwrap().theme())
                    ],
                    // LAYER 1: BIBLE TEXT

                    gtk::ScrolledWindow {
                        set_vexpand: true,
                        set_hscrollbar_policy: gtk::PolicyType::Never,
                        #[local_ref]
                        section_list -> gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_margin_all: 30,
                        },
                    },

                    // LAYER 2: BACKGROUND DIMMING
                    #[name = "dim_scrim"]
                    add_overlay = &gtk::Box {
                        add_css_class: "dim-scrim",
                        set_visible: false,
                        set_can_target: false,
                    },

                    // LAYER 3: THE MENU (PINNED TO BOTTOM-RIGHT)
                    #[name = "overlay_container"]
                    add_overlay = &gtk::Box {
                        set_halign: gtk::Align::End,
                        set_valign: gtk::Align::End,
                        set_margin_all: 25,
                        // Ensure this outer box doesn't grow taller than its content
                        set_vexpand: false,

                        #[name = "menu_card"]
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            add_css_class: "page-menu-card",
                            //add_css_class: "osd",
                            set_spacing: 0,
                            set_valign: gtk::Align::End,


                            // TOP ELEMENT: THE BUTTON
                            #[name = "menu_button"]
                            gtk::Button {
                                add_css_class: "circular",
                                add_css_class: "osd",
                                add_css_class: "studypage-menu-trigger-btn",
                                set_has_frame: false,
                                set_width_request: 64,
                                set_height_request: 64,
                                set_halign: gtk::Align::Center,
                                set_valign: gtk::Align::Start,

                                gtk::Image {
                                    set_icon_name: Some("page-menu-symbolic"),
                                    set_pixel_size: 24,
                                }
                            },

                            // BOTTOM ELEMENT: THE REVEALER
                            #[name = "options_revealer"]
                            gtk::Revealer {
                                set_transition_type: gtk::RevealerTransitionType::SlideDown,
                                set_transition_duration: 250,
                                set_visible: false,
                                // This ensures it grows DOWN from the button without pre-allocating space
                                set_valign: gtk::Align::Start,
                                set_vexpand: false,

                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 10,
                                    set_width_request: 350,
                                    // Strict margins: Top is 0 to touch the button


                                    // SECTION: FONT SIZE
                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,
                                        set_hexpand: false,
                                        set_spacing: 12,
                                        add_css_class: "osd",
                                        add_css_class: "studypage-menu-section-container",
                                        set_width_request: 350,

                                        gtk::Label{
                                            set_label: "Theme and Font",
                                            add_css_class: "title-1",
                                            set_margin_start: 20,
                                            set_margin_end: 20,
                                            set_margin_top: 20,
                                            set_xalign: 0.0,
                                        },

                                        gtk::Label{
                                            set_label: "Font Size",
                                            add_css_class: "title-5",
                                            set_margin_start: 20,
                                            set_margin_end: 20,
                                            set_xalign: 0.0,
                                        },

                                        gtk::Box{
                                            set_margin_start: 20,
                                            set_margin_end: 20,
                                            set_orientation: gtk::Orientation::Horizontal,

                                            gtk::Image {
                                                set_icon_name: Some("font-letter-symbolic"),
                                            },

                                            gtk::Scale::with_range(gtk::Orientation::Horizontal, 12.0, 32.0, 1.0) {
                                                set_hexpand: true,
                                                add_css_class: "accent",
                                                #[watch]
                                                set_value: model.config.read().unwrap().font_size(),
                                                connect_value_changed[sender] => move |scale| {
                                                    sender.input(
                                                        StudyInput::ToggleDisplay(
                                                            VerseInputMessage::ChangeFontSize(
                                                                scale.value()
                                                            )
                                                        )
                                                    )
                                                }
                                            },

                                            gtk::Image {
                                                set_icon_name: Some("font-letter-symbolic"),
                                                set_pixel_size: 30,
                                            },
                                        },

                                        gtk::Box{
                                            set_margin_start: 20,
                                            set_margin_bottom: 20,
                                            set_margin_end: 20,
                                            set_orientation: gtk::Orientation::Vertical,
                                            set_hexpand: true,

                                            gtk::Label{
                                                set_label: "Fonts",
                                                add_css_class: "title-5",
                                                set_xalign: 0.0,
                                            },

                                            gtk::ScrolledWindow {
                                                set_hscrollbar_policy: gtk::PolicyType::Automatic,
                                                set_vscrollbar_policy: gtk::PolicyType::Never,
                                                set_hexpand: true,
                                                set_height_request: 40,
                                                add_css_class: "font-scroll-container",

                                                #[name="menu_fonts_container"]
                                                gtk::Box {
                                                    set_orientation: gtk::Orientation::Horizontal,
                                                    set_spacing: 10,
                                                    set_margin_all: 5,
                                                    set_hexpand: true,
                                                }
                                            },

                                           gtk::Button {
                                                set_margin_top: 5,
                                                adw::ButtonContent {
                                                    set_icon_name: "emblem-system-symbolic",
                                                    set_label: "Customize",
                                                },

                                                connect_clicked[sender] => move |_|{
                                                    sender.input(StudyInput::OpenCustomizethemePopup)
                                                }
                                            }
                                        }
                                    },

                                    // SECTION: TOGGLES
                                    gtk::Box{
                                        add_css_class: "osd",
                                        add_css_class: "studypage-menu-section-container",
                                        set_orientation: gtk::Orientation::Vertical,
                                        set_spacing: 12,
                                        set_width_request: 350,

                                        gtk::Label{
                                            set_label: "Book Options",
                                            add_css_class: "title-1",
                                            set_margin_start: 20,
                                            set_margin_end: 20,
                                            set_margin_top: 20,
                                            set_xalign: 0.0,
                                        },


                                        gtk::Box {
                                            set_orientation: gtk::Orientation::Vertical,
                                            set_spacing: 10,
                                            set_homogeneous: false,
                                            set_margin_end: 20,
                                            set_margin_start: 20,

                                            gtk::Label{
                                                set_label: "Text",
                                                add_css_class: "title-5",
                                                set_xalign: 0.0,
                                            },

                                             gtk::FlowBox {
                                                set_orientation: gtk::Orientation::Horizontal,
                                                set_hexpand: true,
                                                set_max_children_per_line: 1,
                                                set_min_children_per_line: 1,


                                                gtk::CheckButton {
                                                    set_label: Some("Words of Christ in Red"),
                                                    #[watch]
                                                    set_active: model.config.read().unwrap().christ_words_red(),
                                                    connect_toggled[sender] => move |btn| {
                                                        let msg = VerseInputMessage::PutChristWordsInRed(btn.is_active());
                                                        sender.input(StudyInput::ToggleDisplay(msg));
                                                    }
                                                },

                                                gtk::Box{
                                                    set_orientation: gtk::Orientation::Horizontal,
                                                    set_spacing: 30,

                                                    gtk::Label{
                                                        set_label: "Added words"
                                                    },

                                                    gtk::Separator{
                                                        set_orientation: gtk::Orientation::Horizontal,
                                                        add_css_class: "spacer",
                                                        set_hexpand: true
                                                    },

                                                    gtk::DropDown {
                                                        set_halign: gtk::Align::End,
                                                        set_model: Some(&gtk::StringList::new(
                                                            &AddedWordStyle::all().iter().map(
                                                                |style| style.to_string()
                                                            ).collect::<Vec<_>>()
                                                        .iter().map(
                                                            |string| string.as_str()
                                                        ).collect::<Vec<_>>())),
                                                        connect_selected_item_notify[sender] => move |dd| {
                                                            //sender.input(StudyPageInput::UpdateModule(dd.selected()));
                                                        }
                                                    },
                                                }

                                             },

                                            gtk::Label{
                                                set_label: "Lexicons",
                                                add_css_class: "title-5",
                                                set_xalign: 0.0,
                                            },


                                            gtk::FlowBox {
                                                set_orientation: gtk::Orientation::Horizontal,
                                                set_hexpand: true,
                                                set_max_children_per_line: 2,
                                                set_min_children_per_line: 2,


                                                gtk::CheckButton {
                                                    set_label: Some("Strongs"),
                                                    #[watch]
                                                    set_active: model.config.read().unwrap().show_strongs(),
                                                    connect_toggled[sender] => move |btn| {
                                                        let msg = if btn.is_active() { VerseInputMessage::EnableStrongs }
                                                                else { VerseInputMessage::DisableStrongs };
                                                        sender.input(StudyInput::ToggleDisplay(msg));
                                                    }
                                                },

                                                 gtk::CheckButton {
                                                    set_label: Some("Lemma"),
                                                    #[watch]
                                                    set_active: model.config.read().unwrap().show_lemma(),
                                                    connect_toggled[sender] => move |btn| {
                                                        let msg = if btn.is_active() { VerseInputMessage::EnableLemma }
                                                                else { VerseInputMessage::DisableLemma };
                                                        sender.input(StudyInput::ToggleDisplay(msg));
                                                    }
                                                },
                                                gtk::CheckButton {
                                                    set_label: Some("Morph"),
                                                    #[watch]
                                                    set_active: model.config.read().unwrap().show_morphs(),
                                                    connect_toggled[sender] => move |btn| {
                                                        let msg = if btn.is_active() { VerseInputMessage::EnableMorphs }
                                                                else { VerseInputMessage::DisableMorphs };
                                                        sender.input(StudyInput::ToggleDisplay(msg));
                                                    }
                                                },


                                            },
                                        },
                                        gtk::Box {
                                                set_spacing: 8,
                                                set_homogeneous: true,
                                                set_margin_end: 20,
                                                set_margin_start: 20,
                                                set_margin_bottom: 20,

                                                 gtk::CheckButton {
                                                    set_label: Some("Show verse Notes"),
                                                    #[watch]
                                                    set_active: model.config.read().unwrap().show_notes(),
                                                    connect_toggled[sender] => move |btn| {
                                                        let msg = if btn.is_active() { VerseInputMessage::EnableNotes }
                                                                else { VerseInputMessage::DisableNotes };
                                                        sender.input(StudyInput::ToggleDisplay(msg));
                                                    }
                                                },
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
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (engine, module, query) = init;
        let section_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let sections = FactoryVecDeque::builder()
            .launch(section_container)
            .forward(sender.output_sender(), move |message| match message {
                SectionOutput::Lookup(query) => StudyPageOutput::LookupSelectedStrong(query),
            });

        let modules = engine.get_bible_modules();

        let module = modules.first().unwrap();

        let model = BiblePage {
            engine,
            module: module.clone(),
            sections,
            config: Arc::new(RwLock::new(PageDisplayConfig::new())),
            customize_theme_popup: None,
            annotations: AnnotationSettings::load_all(),
        };

        let section_list = model.sections.widget();
        let widgets = view_output!();
        let motion = gtk::EventControllerMotion::new();

        let options_revealer = &widgets.options_revealer;
        let dim_scrim = &widgets.dim_scrim;
        let menu_button = &widgets.menu_button;

        motion.connect_enter(clone!(
            #[weak]
            options_revealer,
            #[weak]
            dim_scrim,
            #[weak]
            menu_button,
            move |_, _, _| {
                options_revealer.set_visible(true);
                options_revealer.set_reveal_child(true);
                dim_scrim.set_visible(true);
                menu_button.set_opacity(0.0);
                menu_button.set_can_target(false);
            }
        ));

        motion.connect_leave(clone!(
            #[weak]
            options_revealer,
            #[weak]
            dim_scrim,
            #[weak]
            menu_button,
            move |_| {
                options_revealer.set_reveal_child(false);
                dim_scrim.set_visible(false);

                menu_button.set_opacity(1.0);
                menu_button.set_can_target(true);
            }
        ));

        options_revealer.connect_child_revealed_notify(move |rev| {
            if !rev.reveals_child() && !rev.is_child_revealed() {
                rev.set_visible(false);
            }
        });

        widgets.overlay_container.add_controller(motion);

        let menu_fonts_container = widgets.menu_fonts_container.clone();
        model.populate_fonts_container(&menu_fonts_container, sender.clone());

        sender.input(StudyInput::LoadReference(query));
        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            StudyInput::LoadReference(refe) => self.load_reference(&refe),
            StudyInput::OpenCustomizethemePopup => {
                let controller = CustomizeThemePopup::builder()
                    .launch((self.config.clone(), self.engine.clone()))
                    .forward(sender.input_sender(), move |msg| match msg {
                        CustomizeThemeOutput::Close => StudyInput::CloseCustomizethemePopup,
                        CustomizeThemeOutput::SaveConfig(config) => StudyInput::SetConfig(config),
                    });

                controller.widget().present();
                self.customize_theme_popup = Some(controller);
            }
            StudyInput::CloseCustomizethemePopup => {
                if let Some(controller) = self.customize_theme_popup.take() {
                    controller.widget().close();
                }
            }
            // StudyInput::SelectStrong(_) => {}
            //StudyInput::SetModule(name) => self.module = name,
            StudyInput::ToggleDisplay(factory_msg) => {
                self.config.write().unwrap().apply_message(&factory_msg);
                for i in 0..self.sections.len() {
                    self.sections
                        .send(i, SectionInput::ToggleDisplay(factory_msg.clone()));
                }
                let theme = self.config.read().unwrap().theme();
                let new_class = self.make_css_preview_clss(theme.clone());

                //remove old theme classes first
                widgets
                    .page_overlay
                    .remove_css_class("preview-area-Classic");
                widgets.page_overlay.remove_css_class("preview-area-Modern");
                widgets
                    .page_overlay
                    .remove_css_class("preview-area-Default");
                widgets
                    .page_overlay
                    .remove_css_class("preview-area-Compact");

                // add the new one
                widgets.page_overlay.add_css_class(&new_class);

                let _ = sender.output(StudyPageOutput::ChangeTheme);
                self.populate_fonts_container(&widgets.menu_fonts_container, sender);
            }
            StudyInput::SetConfig(config) => {
                sender.input(StudyInput::ToggleDisplay(
                    VerseInputMessage::UpdateDisplayConf(config),
                ));
                self.populate_fonts_container(&widgets.menu_fonts_container, sender);
            }
            _ => {}
        }
    }
}
