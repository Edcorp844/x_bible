use adw::prelude::*;
use relm4::prelude::*;
use xbible_engine::engines::module_engine::module_engine_extensions::{module_engine_dictionary_ext::DictionaryQuery, module_engine_module_content_ext::Verse};

use crate::features::{
    bible::components::page::{
        helpers::AvailableFonts,
        verse_components::{
            annotation_colors::{AnnotationColor, AnnotationOutput},
            verse_annotation::{AnnotationSettings, VerseAnnotation},
            verse_menu_button::VerseMenuButton,
        },
        word::{WordModel, WordModelInput, WordModelOutput},
    },
    core::{
        display_configurations::Config::TextConfig,
        
    },
};

pub struct VerseModel {
    pub data: Verse,
    pub config: TextConfig,
    pub word_controllers: Vec<Controller<WordModel>>,
    pub text_direction: gtk::TextDirection,
    pub annotation: VerseAnnotation,
}

#[derive(Debug, Clone)]
pub enum VerseInputMessage {
    UpdateDisplayConf(TextConfig),
    EnableStrongs,
    DisableStrongs,
    EnableNotes,
    DisableNotes,
    EnableMorphs,
    DisableMorphs,
    EnableLemma,
    DisableLemma,
    ChangeFontSize(f64),
    ChangeFont(AvailableFonts),
    ChangeWordSpacing(f64),
    ChangeLineSpacing(f64),
    ChangeBoldFont(bool),
    ChangeJustify(bool),
    PutChristWordsInRed(bool),
    LookUp(DictionaryQuery),
    OpenMenu { x: f64, y: f64 },

    //======verse annotaions messages=====
    Highlight(String),
}

#[derive(Debug, Clone)]
pub enum VerseOutputMessage {
    Lookup(DictionaryQuery),
}

// --- VERSE FACTORY ---
#[relm4::component(pub)]
impl Component for VerseModel {
    type Init = (Verse, TextConfig, gtk::TextDirection, VerseAnnotation);
    type Input = VerseInputMessage;
    type Output = VerseOutputMessage;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 12,
            set_hexpand: true,
            add_css_class: "verse-root",

            #[name = "verse_popover"]
                gtk::Popover {
                    set_has_arrow: true,
                    set_autohide: true,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_width_request: 380,
                        set_height_request: 150,
                        set_margin_all: 5,

                        gtk::Label{
                            set_xalign: 0.0,
                            set_markup: &format!(
                                "<b>{}</b>",
                            model.data.osis_id.as_str()
                            ),
                        },

                        #[name = "swatch_container"]
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 12,
                            set_halign: gtk::Align::Center,
                            set_hexpand: true,
                            set_margin_all: 12,
                        },

                        #[name = "view_stack"]
                        adw::ViewStack {
                            set_vexpand: true,
                        },

                        #[name = "view_switcher"]
                        adw::InlineViewSwitcher {
                            //set_policy: adw::ViewSwitcherPolicy::Wide,
                            add_css_class: "round",
                        }
                    }
                },

            add_controller = gtk::GestureClick {
                set_button: 3, // 3 is Right Click
                connect_released[sender] => move |gesture, _, x, y| {
                    gesture.set_state(gtk::EventSequenceState::Claimed);
                    // Send the coordinates to the update function
                    sender.input(VerseInputMessage::OpenMenu { x, y });
                }
            },

            // 1. Verse Number - Aligned to the top to stay fixed
            gtk::Label {
                add_css_class: "verser-number",
                set_visible: model.data.number != 0,
                set_markup: &format!(
                    "<span size='large'>{}</span>",
                    model.data.number
                ),
                set_valign: gtk::Align::Start,
                set_visible: model.text_direction == gtk::TextDirection::Ltr,
            },

            // 2. Content Stack (Text + Notes)
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 8, // Tight spacing between text and notes
                set_hexpand: true,

                // Main Bible Text
                #[local_ref]
                word_flow -> adw::WrapBox {
                    #[watch]
                    set_line_spacing: model.config.read().unwrap().pango_line_spacing(),
                    set_hexpand: true,
                    #[watch]
                    set_justify: if model.config.read().unwrap().justify() {adw::JustifyMode::Fill} else{adw::JustifyMode::None},
                    set_halign: gtk::Align::Start,
                    #[watch]
                    set_direction: model.text_direction,
                },

                // Notes Revealer - Animates expansion when show_notes is true
                gtk::Revealer {
                    set_transition_type: gtk::RevealerTransitionType::SlideDown,
                    set_transition_duration: 350,

                    #[watch]
                    set_reveal_child: !model.data.notes.is_empty() && model.config.read().unwrap().show_notes(),

                    #[watch]
                    set_visible: !model.data.notes.is_empty(),

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_margin_top: 4,

                        #[local_ref]
                        notes_container -> adw::WrapBox {
                            add_css_class: "verse-notes",
                            set_hexpand: true,
                        }
                    }
                }
            },

             gtk::Label {
                add_css_class: "verser-number",
                set_visible: model.data.number != 0,
                set_markup: &format!(
                    "<span size='large'>{}</span>",
                    model.data.number
                ),
                set_valign: gtk::Align::Start,
                set_visible: model.text_direction == gtk::TextDirection::Rtl,
            },
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (data, config, text_direction, annotation) = init;

        let mut model = Self {
            data,
            config,
            word_controllers: Vec::new(),
            text_direction,
            annotation: annotation.clone(),
        };

        let mut word_controllers = Vec::new();
        let word_flow_box = adw::WrapBox::builder()
            .line_spacing(6)
            .hexpand(true)
            .halign(gtk::Align::Start)
            .build();

        for word in &model.data.words {
            let controller = WordModel::builder()
                .launch((
                    word.clone(),
                    model.config.clone(),
                    model.text_direction,
                    annotation.clone(),
                ))
                .forward(sender.input_sender(), move |message| match message {
                    WordModelOutput::LookUp(text) => VerseInputMessage::LookUp(text),
                });

            word_flow_box.append(controller.widget());
            word_controllers.push(controller);
        }

        model.word_controllers = word_controllers;

        let word_flow = &word_flow_box;
        let notes_container = adw::WrapBox::builder()
            .line_spacing(6)
            .hexpand(true)
            .halign(gtk::Align::Start)
            .build();
        for word in model.data.notes.clone() {
            let note_label = gtk::Label::builder()
                .wrap(true)
                .margin_end(16)
                .css_classes(vec!["verse-note"])
                .build();
            note_label.set_markup(
                format!("<span size='large' foreground='#d71452'><span size='small' foreground='#c314d7'><i>Note on Verse {}: </i></span><i>{}</i></span>",model.data.number, word).as_str(),
            );

            notes_container.append(&note_label);
        }

        let widgets = view_output!();

        let colors = vec![
            "transparent",
            "var(--blue-3)",
            "var(--yellow-3)",
            "var(--green-3)",
            "var(--orange-3)",
            "var(--red-3)",
            "var(--purple-3)",
            "var(--brown-3)",
        ];

        for hex in colors {
            let swatch = AnnotationColor::builder().launch(hex.to_string()).forward(
                sender.input_sender(),
                |output| match output {
                    AnnotationOutput::Selected(color) => VerseInputMessage::Highlight(color),
                },
            );

            // swatch_container is a gtk::Box you defined for your stack page
            widgets.swatch_container.append(swatch.widget());
        }

        // --- CONNECT WIDGETS MANUALLY HERE ---
        // This avoids the 'unrecognized identifier' error in the view macro
        widgets.view_switcher.set_stack(Some(&widgets.view_stack));

        // Adding Stack Pages
        widgets.view_stack.add_titled_with_icon(
            &Self::make_annotation_menu(),
            Some("lex"),
            "Annotate",
            "accessories-dictionary-symbolic",
        );
        widgets.view_stack.add_titled_with_icon(
            &Self::make_study_menu(),
            Some("study"),
            "Study",
            "emblem-documents-symbolic",
        );
        widgets.view_stack.add_titled_with_icon(
            &Self::make_share_menu(),
            Some("share"),
            "Share",
            "emblem-shared-symbolic",
        );

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
            VerseInputMessage::OpenMenu { x, y } => {
                widgets
                    .verse_popover
                    .set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
                widgets.verse_popover.popup();
            }
            VerseInputMessage::LookUp(query) => {
                let _ = sender.output(VerseOutputMessage::Lookup(query));
            }

            // ===== verse menu ======
            VerseInputMessage::Highlight(color) => {
                println!("hightlighting {} with color {}", self.data.osis_id, color);
                self.annotation.color = Some(color);
                AnnotationSettings::save_verse(&self.data.osis_id, self.annotation.clone());

                for controller in &self.word_controllers {
                    controller.emit(WordModelInput::UpdateAnnotation(self.annotation.clone()));
                }
            }
            _ => {
                self.config.write().unwrap().apply_message(&message);

                for controller in &self.word_controllers {
                    controller.emit(WordModelInput::UpdateConfig(self.config.clone()));
                }
            }
        }
    }
}

impl VerseModel {
    fn make_annotation_menu() -> gtk::Box {
        // 1. Create the container with centering and expansion
        let container = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(10) // Breathable space between buttons
            .halign(gtk::Align::Center)
            .hexpand(true)
            .margin_top(10)
            .margin_bottom(10)
            .build();

        // 2. Define your options (Label, Icon Name)
        let options = vec![
            ("Note", "document-new-symbolic"), // Standard for a text note/annotation
            ("Tag", "mail-attachment-symbolic"), // Distinct tag shape
            ("Link", "insert-link-symbolic"),  // Standard chain link icon
            ("Bookmark", "bookmark-new-symbolic"), // Keep this, it's correct
        ];

        for (label, icon) in options {
            // 3. Launch the component
            let controller = VerseMenuButton::builder()
                .launch((label.to_string(), icon.to_string()))
                .detach(); // Detach since we aren't handling internal messages here yet

            let widget = controller.widget();

            // 4. Ensure each button assembly takes up even space
            widget.set_hexpand(true);
            widget.set_halign(gtk::Align::Center);

            container.append(widget);
        }

        container
    }

    fn make_share_menu() -> gtk::Box {
        // 1. Create the container with centering and expansion
        let container = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(10) // Breathable space between buttons
            .halign(gtk::Align::Center)
            .hexpand(true)
            .margin_top(10)
            .margin_bottom(10)
            .build();

        // 2. Define your options (Label, Icon Name)
        let options = vec![
            ("Copy", "edit-copy-symbolic"),
            ("Share", "emblem-shared-symbolic"),
            ("Select", "selection-mode-symbolic"),
        ];

        for (label, icon) in options {
            // 3. Launch the component
            let controller = VerseMenuButton::builder()
                .launch((label.to_string(), icon.to_string()))
                .detach(); // Detach since we aren't handling internal messages here yet

            let widget = controller.widget();

            // 4. Ensure each button assembly takes up even space
            widget.set_hexpand(true);
            widget.set_halign(gtk::Align::Center);

            container.append(widget);
        }

        container
    }

    fn make_study_menu() -> gtk::Box {
        // 1. Create the container with centering and expansion
        let container = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(10) // Breathable space between buttons
            .halign(gtk::Align::Center)
            .hexpand(true)
            .margin_top(10)
            .margin_bottom(10)
            .build();

        // 2. Define your options (Label, Icon Name)
        let options = vec![
            ("Note", "document-new-symbolic"), // Standard for a text note/annotation
            ("Tag", "mail-attachment-symbolic"), // Distinct tag shape
            ("Link", "insert-link-symbolic"),  // Standard chain link icon
            ("Bookmark", "bookmark-new-symbolic"), // Keep this, it's correct
        ];

        for (label, icon) in options {
            // 3. Launch the component
            let controller = VerseMenuButton::builder()
                .launch((label.to_string(), icon.to_string()))
                .detach(); // Detach since we aren't handling internal messages here yet

            let widget = controller.widget();

            // 4. Ensure each button assembly takes up even space
            widget.set_hexpand(true);
            widget.set_halign(gtk::Align::Center);

            container.append(widget);
        }

        container
    }
}
