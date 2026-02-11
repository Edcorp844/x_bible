use adw::prelude::*;
use ego_tree::NodeRef;
use gtk::glib::clone;
use relm4::prelude::*;
use std::sync::Arc;

use crate::features::bible::components::page::verse::{
    DisplayConfig, VerseInputMessage, VerseModel,
};
use crate::{
    features::{
        bible::components::page::helpers::{LexicalInfo, SegmentStyle, Verse, Word},
        core::module_engine::sword_engine::SwordEngine,
    },
    sword_sys::*,
};
pub struct BiblePage {
    pub mgr_ptr: isize,
    module: String,
    verses: FactoryVecDeque<VerseModel>,
}

#[derive(Debug)]
pub enum StudyInput {
    LoadReference(String),
    SelectStrong(String),
    SetModule(String),
    /// Sends a toggle message to all verses in the factory
    ToggleDisplay(VerseInputMessage),
}
#[relm4::component(pub)]
impl SimpleComponent for BiblePage {
    type Init = (Arc<SwordEngine>, String, String);
    type Input = StudyInput;
    type Output = ();

    view! {
        adw::NavigationPage {
            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                #[name="page_overlay"]
                gtk::Overlay {
                    set_vexpand: true,
                    add_css_class: "page-overlay",

                    // LAYER 1: BIBLE TEXT
                    gtk::ScrolledWindow {
                        set_vexpand: true,
                        set_hscrollbar_policy: gtk::PolicyType::Never,
                        #[local_ref]
                        verse_list -> gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_margin_all: 30,
                            set_spacing: 8,
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
                            add_css_class: "osd",
                            set_spacing: 0,
                            set_valign: gtk::Align::End,

                            // TOP ELEMENT: THE BUTTON
                            #[name = "menu_button"]
                            gtk::Button {
                                add_css_class: "circular",
                                add_css_class: "liquid-trigger",
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
                                    set_spacing: 20,
                                    set_width_request: 250,
                                    // Strict margins: Top is 0 to touch the button
                                    set_margin_top: 0,
                                    set_margin_bottom: 20,
                                    set_margin_start: 20,
                                    set_margin_end: 20,

                                    // SECTION: FONT SIZE
                                    gtk::Box {
                                        set_spacing: 12,
                                        gtk::Image {
                                            set_icon_name: Some("font-letter-symbolic"),
                                        },
                                        gtk::Scale::with_range(gtk::Orientation::Horizontal, 12.0, 32.0, 1.0) {
                                            set_hexpand: true,
                                            add_css_class: "accent",
                                            set_value: 16.0,
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

                                    // SECTION: TOGGLES
                                    gtk::Box {
                                        set_spacing: 10,
                                        set_homogeneous: true,

                                        gtk::CheckButton {
                                            set_label: Some("Strongs"),
                                            add_css_class: "pill",
                                            connect_toggled[sender] => move |btn| {
                                                let msg = if btn.is_active() { VerseInputMessage::EnableStrongs }
                                                          else { VerseInputMessage::DisableStrongs };
                                                sender.input(StudyInput::ToggleDisplay(msg));
                                            }
                                        },

                                        gtk::CheckButton {
                                            set_label: Some("Notes"),
                                            add_css_class: "pill",
                                            connect_toggled[sender] => move |btn| {
                                                let msg = if btn.is_active() { VerseInputMessage::EnableNotes }
                                                          else { VerseInputMessage::DisableNotes };
                                                sender.input(StudyInput::ToggleDisplay(msg));
                                            }
                                        },
                                    },
                                     gtk::Box {
                                            set_spacing: 8,
                                            set_homogeneous: true,
                                            gtk::CheckButton {
                                                set_label: Some("Lemma"),
                                                add_css_class: "pill",
                                                connect_toggled[sender] => move |btn| {
                                                    let msg = if btn.is_active() { VerseInputMessage::EnableLemma }
                                                              else { VerseInputMessage::DisableLemma };
                                                    sender.input(StudyInput::ToggleDisplay(msg));
                                                }
                                            },
                                            gtk::CheckButton {
                                                set_label: Some("Morph"),
                                                add_css_class: "pill",
                                                connect_toggled[sender] => move |btn| {
                                                    let msg = if btn.is_active() { VerseInputMessage::EnableMorphs }
                                                              else { VerseInputMessage::DisableMorphs };
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

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (engine, module, query) = init;
        let verse_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
        let verses = FactoryVecDeque::builder().launch(verse_container).detach();
        let mgr_ptr = engine.inner.lock().unwrap().mgr;

        let model = BiblePage {
            mgr_ptr,
            module: module.clone(),
            verses,
        };

        let verse_list = model.verses.widget();
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

                // Button fades out as menu takes over
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

                // Button reappears
                menu_button.set_opacity(1.0);
                menu_button.set_can_target(true);
            }
        ));

        // Cleanup layout after animation finishes
        options_revealer.connect_child_revealed_notify(move |rev| {
            if !rev.reveals_child() && !rev.is_child_revealed() {
                rev.set_visible(false);
            }
        });

        widgets.overlay_container.add_controller(motion);

        sender.input(StudyInput::LoadReference(query));
        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            StudyInput::LoadReference(refe) => self.load_reference(&refe),
            StudyInput::SelectStrong(_) => {}
            StudyInput::SetModule(name) => self.module = name,
            StudyInput::ToggleDisplay(factory_msg) => {
                for i in 0..self.verses.len() {
                    self.verses.send(i, factory_msg.clone());
                }
            }
        }
    }
}

impl BiblePage {
    pub fn load_reference(&mut self, reference: &str) {
        let verses = self.render_content_to_verses(reference);
        let mut guard = self.verses.guard();
        guard.clear();
        for verse in verses {
            guard.push_back((
                verse,
                DisplayConfig {
                    show_strongs: false,
                    show_morphs: false,
                    show_lemma: false,
                    show_notes: false,
                    added_style: super::word::AddedWordStyle::Brackets,
                },
            ));
        }
    }

    pub fn render_content_to_verses(&self, reference: &str) -> Vec<Verse> {
        use std::ffi::CString;
        let mut verses = Vec::new();

        unsafe {
            let key_ref = CString::new(reference).unwrap();
            let module_name = CString::new(self.module.as_str()).unwrap();

            let options = [
                "Strong's Numbers",
                "Morphological Tags",
                "Footnotes",
                "Cross-references",
            ];
            let on = CString::new("On").unwrap();

            for opt in options {
                let opt_c = CString::new(opt).unwrap();
                org_crosswire_sword_SWMgr_setGlobalOption(
                    self.mgr_ptr,
                    opt_c.as_ptr(),
                    on.as_ptr(),
                );
            }

            let h_mod =
                org_crosswire_sword_SWMgr_getModuleByName(self.mgr_ptr, module_name.as_ptr());
            if h_mod == 0 {
                return verses;
            }

            org_crosswire_sword_SWModule_setKeyText(h_mod, key_ref.as_ptr());
            let initial_key = self
                .sword_ptr_to_string(org_crosswire_sword_SWModule_getKeyText(h_mod))
                .unwrap_or_default();

            let chapter_boundary = initial_key
                .split(|c| c == ':' || c == '.')
                .next()
                .unwrap_or(&initial_key)
                .to_string();

            loop {
                let key = match self
                    .sword_ptr_to_string(org_crosswire_sword_SWModule_getKeyText(h_mod))
                {
                    Some(k) => k,
                    None => break,
                };
                if !key.starts_with(&chapter_boundary) {
                    break;
                }

                let raw_osis = match self
                    .sword_ptr_to_string(org_crosswire_sword_SWModule_getRawEntry(h_mod))
                {
                    Some(s) => s,
                    None => break,
                };

                // Debug print raw OSIS
                println!("[+] {}\n", raw_osis);

                let (mut words, notes) = self.parse_osis_content(&raw_osis);

                // Apply grouping markers (brackets for Added, potential spans for Red)
                self.apply_group_metadata(&mut words);

                verses.push(Verse {
                    osis_id: key.clone(),
                    number: self.extract_verse_number(&key),
                    words,
                    notes,
                    is_paragraph_start: raw_osis.contains("type=\"paragraph\"")
                        || key.ends_with(":1"),
                });

                org_crosswire_sword_SWModule_next(h_mod);
                if org_crosswire_sword_SWModule_popError(h_mod) != 0 {
                    break;
                }
            }
        }
        verses
    }

    fn parse_osis_content(&self, osis: &str) -> (Vec<Word>, Vec<String>) {
        use scraper::Html;
        let fragment = Html::parse_fragment(osis);
        let mut words = Vec::new();
        let mut verse_notes = Vec::new();

        self.walk_osis(
            fragment.tree.root(),
            &mut words,
            &mut Vec::new(),
            None,
            false, // is_red (Jesus block)
            false, // is_added (Theological status)
            false, // is_italic (General style)
            false, // is_inside_note
        );

        let note_selector = scraper::Selector::parse("note").unwrap();
        let catch_selector = scraper::Selector::parse("catchWord").unwrap();

        for note_node in fragment.select(&note_selector) {
            let el = note_node.value();
            let note_type = el.attr("type").unwrap_or("");
            let osis_ref = el.attr("osisRef").unwrap_or("");
            let full_note_text = note_node.text().collect::<Vec<_>>().join(" ");

            if note_type == "crossReference" || !osis_ref.is_empty() {
                let cross_ref_data = if !osis_ref.is_empty() {
                    format!("[Cross-Ref: {}] {}", osis_ref, full_note_text)
                } else {
                    full_note_text.clone()
                };
                verse_notes.push(cross_ref_data);
                continue;
            }

            if let Some(catch_node) = note_node.select(&catch_selector).next() {
                let clean_catch = catch_node
                    .text()
                    .collect::<String>()
                    .replace("…", "")
                    .to_lowercase()
                    .trim()
                    .to_string();

                let mut attached = false;
                for word in words.iter_mut() {
                    if word.text.to_lowercase().contains(&clean_catch) {
                        word.note = Some(full_note_text.clone());
                        attached = true;
                        break;
                    }
                }
                if !attached {
                    verse_notes.push(full_note_text);
                }
            } else {
                verse_notes.push(full_note_text);
            }
        }
        (words, verse_notes)
    }

    fn walk_osis(
        &self,
        node: NodeRef<scraper::node::Node>,
        words: &mut Vec<Word>,
        _notes: &mut Vec<String>,
        parent_lex: Option<LexicalInfo>,
        is_red: bool,
        is_added: bool,
        is_italic: bool,
        is_inside_note: bool,
    ) {
        use scraper::node::Node;

        match node.value() {
            Node::Text(t) => {
                if !is_inside_note {
                    let text = t.text.trim();
                    if !text.is_empty() {
                        for piece in text.split_whitespace() {
                            words.push(Word {
                                text: piece.to_string(),
                                // This determines if we see "Plain" or "Added" in your debug
                                style: if is_added {
                                    SegmentStyle::Added
                                } else {
                                    SegmentStyle::Plain
                                },
                                is_red,
                                is_italic,
                                is_bold_text: false,
                                lex: parent_lex.clone(),
                                note: None,
                                is_first_in_group: false,
                                is_last_in_group: false,
                                is_punctuation: piece.chars().all(|c| c.is_ascii_punctuation()),
                            });
                        }
                    }
                }
            }
            Node::Element(el) => {
                // Inheritance: start with the parent's state
                let mut current_lex = parent_lex.clone();
                let mut active_red = is_red;
                let mut active_added = is_added;
                let mut active_italic = is_italic;
                let mut active_note = is_inside_note;

                match el.name() {
                    "w" => {
                        let raw_lemma = el.attr("lemma").unwrap_or("");
                        let raw_morph = el.attr("morph").unwrap_or("");

                        let strongs: Vec<String> = raw_lemma
                            .split_whitespace()
                            .filter(|s| s.starts_with("strong:"))
                            .map(|s| s.trim_start_matches("strong:").to_string())
                            .collect();

                        let tr_lemma = raw_lemma
                            .split_whitespace()
                            .find(|s| s.starts_with("lemma.TR:"))
                            .map(|s| s.trim_start_matches("lemma.TR:").to_string());

                        current_lex = Some(LexicalInfo {
                            strongs,
                            lemma: tr_lemma,
                            morph: self.decode_morph(raw_morph),
                            ..Default::default()
                        });
                    }
                    "q" if el.attr("who") == Some("Jesus") => {
                        active_red = true;
                    }
                    "transChange" if el.attr("type") == Some("added") => {
                        active_added = true;
                        //active_italic = true;
                    }
                    "hi" if el.attr("type") == Some("italic") => {
                        active_italic = true;
                    }
                    "note" => {
                        active_note = true;
                    }
                    _ => {}
                }

                // Recurse into children with the UPDATED state
                for child in node.children() {
                    self.walk_osis(
                        child,
                        words,
                        _notes,
                        current_lex.clone(),
                        active_red,
                        active_added,
                        active_italic,
                        active_note,
                    );
                }
            }
            _ => {
                // For non-elements/non-text, just pass the state through
                for child in node.children() {
                    self.walk_osis(
                        child,
                        words,
                        _notes,
                        parent_lex.clone(),
                        is_red,
                        is_added,
                        is_italic,
                        is_inside_note,
                    );
                }
            }
        }
    }

    fn apply_group_metadata(&self, words: &mut [Word]) {
        let len = words.len();
        if len == 0 {
            return;
        }

        for i in 0..len {
            // 1. Check for Added words (Theological status)
            if words[i].style == SegmentStyle::Added {
                let is_prev_added = if i > 0 {
                    words[i - 1].style == SegmentStyle::Added
                } else {
                    false
                };
                let is_next_added = if i < len - 1 {
                    words[i + 1].style == SegmentStyle::Added
                } else {
                    false
                };

                if !is_prev_added {
                    words[i].is_first_in_group = true;
                }
                if !is_next_added {
                    words[i].is_last_in_group = true;
                }
            }
            // 2. Check for Jesus words (is_red) ONLY if not already marked by an Added group
            else if words[i].is_red {
                let is_prev_red = if i > 0 { words[i - 1].is_red } else { false };
                let is_next_red = if i < len - 1 {
                    words[i + 1].is_red
                } else {
                    false
                };

                if !is_prev_red {
                    words[i].is_first_in_group = true;
                }
                if !is_next_red {
                    words[i].is_last_in_group = true;
                }
            }
        }
    }

    fn extract_verse_number(&self, key: &str) -> i32 {
        key.split(|c| c == '.' || c == ':')
            .last()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    fn decode_morph(&self, morph: &str) -> Vec<String> {
        let code = morph.split(':').last().unwrap_or(morph);
        let mut parts = Vec::new(); // Compiler infers Vec<String>

        let chars: Vec<char> = code.chars().collect();
        if chars.is_empty() {
            return Vec::new();
        }

        // Positional Logic: Part of Speech
        match chars[0] {
            'N' => parts.push("Noun".to_string()),
            'V' => parts.push("Verb".to_string()),
            'A' => parts.push("Adj".to_string()),
            'R' => parts.push("Pron".to_string()),
            'D' => parts.push("Adv".to_string()),
            'P' => parts.push("Prep".to_string()),
            'C' => parts.push("Conj".to_string()),
            'I' => parts.push("Interj".to_string()),
            _ => {}
        }

        // Positional logic for grammatical features
        for (i, &c) in chars.iter().enumerate().skip(1) {
            match c {
                '-' => continue,
                'N' => parts.push("Nom".to_string()),
                'G' => parts.push("Gen".to_string()),
                'D' => parts.push("Dat".to_string()),
                'A' => parts.push("Acc".to_string()),
                'S' => parts.push("Sing".to_string()),
                'P' => {
                    if chars[0] == 'V' && i < 3 {
                        parts.push("Pres".to_string());
                    } else {
                        parts.push("Plur".to_string());
                    }
                }
                'M' => parts.push("Masc".to_string()),
                'F' => parts.push("Fem".to_string()),
                'T' => parts.push("Neut".to_string()),
                _ => {}
            }
        }

        parts
    }

    unsafe fn sword_ptr_to_string(&self, ptr: *const std::os::raw::c_char) -> Option<String> {
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() })
    }
}
