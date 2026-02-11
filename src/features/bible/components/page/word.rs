use adw::prelude::*;
use relm4::prelude::*;

use crate::features::bible::components::page::{
    helpers::{SegmentStyle, Word},
    verse::DisplayConfig,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AddedWordStyle {
    Italic,
    Brackets,
}

pub struct WordModel {
    data: Word,
    config: DisplayConfig,
}

#[derive(Debug)]
pub enum WordModelInput {
    UpdateConfig(DisplayConfig),
}

#[relm4::component(pub)]
impl SimpleComponent for WordModel {
    type Init = (Word, DisplayConfig);
    type Input = WordModelInput;
    type Output = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 2,
            set_halign: gtk::Align::Start,

            #[name="word_wrapper"]
            gtk::Box{
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 2,
                set_halign: gtk::Align::Start,


                gtk::Label {
                    add_css_class: "bible-text",
                    set_hexpand: false,
                    set_margin_start: if model.data.is_punctuation { 0 } else { 8 },
                    #[watch]
                    set_markup: model.render_word().as_str(),
                    set_xalign: 0.0,
                },
            },

            gtk::Box{
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 2,
                set_halign: gtk::Align::Start,

                #[watch]
                set_visible: model.config.show_lemma || model.config.show_strongs || model.config.show_morphs,

                gtk::Revealer{
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

                 gtk::Revealer{
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

                 gtk::Revealer{
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
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            data: init.0,
            config: init.1,
        };

        let morph_box = model.get_morphs_widget();

        let widgets = view_output!();

        let word_wrapper = widgets.word_wrapper.clone();

        if model.data.note.is_some() {
            word_wrapper.append(&model.attach_note());
        }

        ComponentParts { model, widgets }
    }

    fn update(&mut self, msg: Self::Input, _sender: ComponentSender<Self>) {
        match msg {
            WordModelInput::UpdateConfig(new_config) => {
                self.config = new_config;
            }
        }
    }
}

impl WordModel {
    fn render_word(&self) -> String {
        let escaped = gtk::glib::markup_escape_text(&self.data.text);

        let mut content = match self.data.style {
            SegmentStyle::Added => match self.config.added_style {
                AddedWordStyle::Italic => format!("<i>{}</i>", escaped),
                AddedWordStyle::Brackets => {
                    let open = if self.data.is_first_in_group { "[" } else { "" };
                    let close = if self.data.is_last_in_group { "]" } else { "" };
                    format!("{open}{escaped}{close}")
                }
            },
            _ => escaped.to_string(),
        };

        if self.data.is_red {
            content = format!("<span color='#e01b24'>{}</span>", content);
        }

        if self.data.is_italic {
            content = format!("<i>{}</i>", content);
        }

        if self.data.is_bold_text {
            content = format!("<b>{}</b>", content);
        }

        format!("<span size='large'>{}</span>", content)
    }

    fn attach_note(&self) -> gtk::Label {
        let note_label = gtk::Label::builder()
            .use_markup(true)
            .hexpand(false)
            .xalign(0.0)
            .build();

        note_label.set_markup("<span color='#d71452' size='x-small'><i>n*</i></span>");

        // --- CURSOR FIX ---
        let motion = gtk::EventControllerMotion::new();
        motion.connect_enter(|motion, _, _| {
            let widget = motion.widget().unwrap();
            widget.set_cursor_from_name(Some("pointer"));
        });

        motion.connect_leave(|motion| {
            let widget = motion.widget().unwrap();
            // Reset the cursor when leaving
            widget.set_cursor(None);
        });

        note_label.add_controller(motion);

        // --- POPUP LOGIC ---
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

        popover_label.set_markup(
            format!(
                "<span color='#d71452'>{}</span>",
                self.data.note.clone().unwrap_or("".to_string())
            )
            .as_str(),
        );
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
        self.config.show_strongs
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
        self.config.show_lemma
            && self
                .data
                .lex
                .as_ref()
                .map_or(false, |lex| lex.lemma.is_some())
    }

    fn get_lemma_markup(&self) -> String {
        let mut mark_up = String::new();
        if let Some(lex) = self.data.lex.clone() {
            if let Some(lemma) = lex.lemma {
                mark_up = format!("<span  size='small' color='#ed10a3'>{}</span>", lemma);
            }
        }

        mark_up
    }

    fn should_reveal_morphs(&self) -> bool {
        self.config.show_morphs
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
}


