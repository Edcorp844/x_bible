use adw::prelude::*;
use relm4::prelude::*;
use xbible_engine::engines::module_engine::module_engine_extensions::{
    module_engine_dictionary_ext::DictionaryQuery, module_engine_module_content_ext::Word,
};

use crate::features::{
    bible::components::page::{
        helpers::{AddedWordStyle, AvailableFonts},
        verse_components::verse_annotation::VerseAnnotation,
    },
    core::display_configurations::config::TextConfig,
};

#[derive(Debug, Clone)]
pub struct WordModel {
    data: Word,
    config: TextConfig,
    text_direction: gtk::TextDirection,
    annotation: VerseAnnotation,
    red_word: bool,
}

#[derive(Debug)]
pub enum WordModelInput {
    LookUp,
    UpdateConfig(TextConfig),
    UpdateAnnotation(VerseAnnotation),
}

#[derive(Debug)]
pub enum WordModelOutput {
    LookUp(DictionaryQuery),
}

#[relm4::component(pub)]
impl SimpleComponent for WordModel {
    type Init = (Word, TextConfig, gtk::TextDirection, VerseAnnotation);
    type Input = WordModelInput;
    type Output = WordModelOutput;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 2,
            #[track(true)]
            set_halign: model.get_align(),

            #[name="word_wrapper"]
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 2,
                #[track(true)]
                set_halign: model.get_align(),

                gtk::Label {
                    add_css_class: "bible-text",
                    set_hexpand: false,
                    #[track(true)]
                    set_margin_start: if model.data.is_punctuation { 0 } else { model.config.read().unwrap().pango_word_spacing() },
                    #[track(true)]
                    set_markup: model.render_word().as_str(),
                    #[track(true)]
                    set_direction: model.text_direction,
                    set_xalign: 0.0,
                    #[watch]
                    inline_css: &format!(
                        "background-color: alpha({}, 0.8); border-radius: 10px; color: {};",
                        model.annotation.color.as_deref().unwrap_or("transparent"),
                        if model.red_word { "var(--red-3, #ed333b)" } else { "var(--window-fg-color)" }
                    ),

                    add_controller = gtk::GestureClick {
                        set_button: 1,
                        connect_released[sender] => move |_, _, _, _| {
                            sender.input(WordModelInput::LookUp)
                        }
                    }
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 2,
                set_halign: gtk::Align::Start,

                #[watch]
                set_visible: model.config.read().unwrap().show_lemma()
                    || model.config.read().unwrap().show_strongs()
                    || model.config.read().unwrap().show_morphs(),

                gtk::Revealer {
                    set_transition_type: gtk::RevealerTransitionType::SlideDown,
                    set_transition_duration: 350,

                    #[watch]
                    set_reveal_child: model.should_reveal_strong(),

                    gtk::Label {
                        add_css_class: "bible-text",
                        add_css_class: "lexical",
                        set_use_markup: true,
                        set_xalign: 0.0,
                        set_margin_start: 4,
                        set_margin_end: 8,

                        #[watch]
                        set_markup: &model.get_strongs_markup(),
                    }
                },

                gtk::Revealer {
                    set_transition_type: gtk::RevealerTransitionType::SlideDown,
                    set_transition_duration: 350,

                    #[watch]
                    set_reveal_child: model.should_reveal_lemma(),

                    gtk::Label {
                        add_css_class: "bible-text",
                        add_css_class: "lexical",
                        set_use_markup: true,
                        set_xalign: 0.0,
                        set_margin_start: 4,
                        set_margin_end: 8,

                        #[watch]
                        set_markup: &model.get_lemma_markup(),
                    }
                },

                gtk::Revealer {
                    set_transition_type: gtk::RevealerTransitionType::SlideDown,
                    set_transition_duration: 350,

                    #[watch]
                    set_reveal_child: model.should_reveal_morphs(),

                    #[local_ref]
                    morph_box -> gtk::Box {},
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let is_christ_red = init.0.is_red && init.1.read().unwrap().christ_words_red();

        let model = Self {
            data: init.0,
            config: init.1,
            text_direction: init.2,
            annotation: init.3,
            red_word: is_christ_red,
        };

        let morph_box = model.get_morphs_widget();
        let widgets = view_output!();

        if model.data.note.is_some() {
            widgets.word_wrapper.append(&model.attach_note());
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, sender: ComponentSender<Self>) {
        match msg {
            WordModelInput::UpdateConfig(new_config) => {
                self.config = new_config;
                self.red_word = self.data.is_red && self.config.read().unwrap().christ_words_red();
            }
            WordModelInput::UpdateAnnotation(annotation) => {
                self.annotation = annotation;
            }
            WordModelInput::LookUp => {
                let mut strongs = Vec::new();
                if let Some(lex) = &self.data.lex {
                    for s in &lex.strongs {
                        strongs.push(s.to_string());
                    }
                }
                let _ = sender.output(WordModelOutput::LookUp(DictionaryQuery {
                    word: self.data.text.clone(),
                    strongs,
                    language: self.data.language.clone(),
                }));
            }
        }
    }
}

impl WordModel {
    fn render_word(&self) -> String {
        let escaped = gtk::glib::markup_escape_text(&self.data.text);
        let mut content = escaped.to_string();

        let config_guard = self.config.read().unwrap();

        // 1. Words of Christ in Red
        if self.data.is_red && config_guard.christ_words_red() {
            content = format!("<span color='#ed333b'>{}</span>", content);
        }

        // 2. Formatting Added Words (Supplied Text)
        if self.data.is_added {
            match config_guard.added_style() {
                AddedWordStyle::Italic => {
                    content = format!("<i>{}</i>", content);
                }
                AddedWordStyle::Brackets => {
                    if self.data.is_first_added && self.data.is_last_added {
                        content = format!("[{}]", content);
                    } else if self.data.is_first_added {
                        content = format!("[{}", content);
                    } else if self.data.is_last_added {
                        content = format!("{}]", content);
                    }
                }
            }
        }

        // 3. Native Italics
        if self.data.is_italic && !self.data.is_added {
            content = format!("<i>{}</i>", content);
        }

        // 4. Bold Settings
        if config_guard.bold_font() || self.data.is_bold_text {
            content = format!("<b>{}</b>", content);
        }

        // 5. Section Titles
        if self.data.is_title {
            content = format!("<span size='large'><b>{}</b></span>", content);
        }

        // 6. Font Family and Text Size Packaging
        match config_guard.font() {
            AvailableFonts::System => {
                format!(
                    "<span size='{}'>{}</span>",
                    config_guard.pango_text_size(),
                    content
                )
            }
            _ => {
                format!(
                    "<span size='{}' face='{}'>{}</span>",
                    config_guard.pango_text_size(),
                    config_guard.font().to_string(),
                    content,
                )
            }
        }
    }

    fn attach_note(&self) -> gtk::Label {
        let note_label = gtk::Label::builder()
            .use_markup(true)
            .hexpand(false)
            .xalign(0.0)
            .build();

        let note_size = (self.config.read().unwrap().pango_text_size() as f64 * 0.6) as i32;

        let note_markup = match self.config.read().unwrap().font() {
            AvailableFonts::System => format!(
                "<span color='#d71452' size='{}'><i>n*</i></span>",
                note_size
            ),
            _ => format!(
                "<span color='#d71452' size='{}'><i>n*</i></span>",
                note_size
            ),
        };

        note_label.set_markup(&note_markup);

        let motion = gtk::EventControllerMotion::new();
        motion.connect_enter(|motion, _, _| {
            if let Some(widget) = motion.widget() {
                widget.set_cursor_from_name(Some("pointer"));
            }
        });

        motion.connect_leave(|motion| {
            if let Some(widget) = motion.widget() {
                widget.set_cursor(None);
            }
        });

        note_label.add_controller(motion);

        let popover = gtk::Popover::builder()
            .css_classes(["bible-note-popover"])
            .autohide(true)
            .build();

        let popover_label = gtk::Label::builder()
            .wrap(true)
            .max_width_chars(30)
            .margin_top(5)
            .margin_bottom(5)
            .margin_start(5)
            .margin_end(5)
            .build();

        popover_label.set_markup(&format!(
            "<span color='#d71452'>{}</span>",
            self.data.note.as_deref().unwrap_or("")
        ));
        popover.set_child(Some(&popover_label));
        popover.set_parent(&note_label);

        let click = gtk::GestureClick::new();
        let p_clone = popover.clone();
        click.connect_released(move |_, _, _, _| {
            p_clone.popup();
        });

        note_label.add_controller(click);
        note_label
    }

    fn should_reveal_strong(&self) -> bool {
        self.config.read().unwrap().show_strongs()
            && self
                .data
                .lex
                .as_ref()
                .map_or(false, |lex| !lex.strongs.is_empty())
    }

    fn get_strongs_markup(&self) -> String {
        if let Some(lex) = self.data.lex.as_ref() {
            lex.strongs
                .iter()
                .map(|s| format!("<span size='small' color='#1086ed'>{}</span>", s))
                .collect::<Vec<_>>()
                .join(" ")
        } else {
            String::new()
        }
    }

    fn should_reveal_lemma(&self) -> bool {
        self.config.read().unwrap().show_lemma()
            && self
                .data
                .lex
                .as_ref()
                .map_or(false, |lex| lex.lemma.is_some())
    }

    fn get_lemma_markup(&self) -> String {
        if let Some(lex) = &self.data.lex {
            if let Some(lemma) = &lex.lemma {
                return format!("<span size='small' color='#ed10a3'>{}</span>", lemma);
            }
        }
        String::new()
    }

    fn should_reveal_morphs(&self) -> bool {
        self.config.read().unwrap().show_morphs()
            && self
                .data
                .lex
                .as_ref()
                .map_or(false, |lex| !lex.morph.is_empty())
    }

    fn get_morphs_widget(&self) -> gtk::Box {
        let wrapper = gtk::Box::builder().css_classes(vec!["lexical"]).build();
        if let Some(lex) = self.data.lex.as_ref() {
            if !lex.morph.is_empty() {
                let wrap_box = adw::WrapBox::builder().child_spacing(3).build();
                for morph in &lex.morph {
                    let label = gtk::Label::builder().label(morph).build();
                    wrap_box.append(&label);
                }
                wrapper.append(&wrap_box);
            }
        }
        wrapper
    }

    fn get_align(&self) -> gtk::Align {
        match self.text_direction {
            gtk::TextDirection::Rtl => gtk::Align::End,
            _ => gtk::Align::Start,
        }
    }
}