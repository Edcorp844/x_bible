use adw::prelude::*;
use ego_tree::NodeRef;
use relm4::prelude::*;
use std::os::raw::c_char;
use std::{ffi::CStr, sync::Arc};

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
    verses: FactoryVecDeque<Verse>,
}

#[derive(Debug)]
pub enum StudyInput {
    LoadReference(String),
    SelectStrong(String),
    SetModule(String),
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
                gtk::ScrolledWindow {
                    set_vexpand: true,
                    set_hscrollbar_policy: gtk::PolicyType::Never,

                    #[name="page_overlay"]
                    gtk::Overlay {
                        add_css_class: "page-overlay",

                        #[local_ref]
                        verse_list -> gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_margin_all: 30,
                            set_spacing: 8,
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
        sender.input(StudyInput::LoadReference(query));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            StudyInput::LoadReference(refe) => self.load_reference(&refe),
            StudyInput::SelectStrong(_) => {}
            StudyInput::SetModule(name) => self.module = name,
        }
    }
}

impl BiblePage {
    pub fn load_reference(&mut self, reference: &str) {
        let verses = self.render_content_to_verses(reference);
        let mut guard = self.verses.guard();
        guard.clear();
        for v in verses {
            guard.push_back(v);
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
                            morph: Some(self.decode_morph(raw_morph)),
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

    fn decode_morph(&self, morph: &str) -> String {
        let code = morph.split(':').last().unwrap_or(morph);
        let mut parts = Vec::new();

        let chars: Vec<char> = code.chars().collect();
        if chars.is_empty() {
            return code.to_string();
        }

        // First char: Part of Speech
        match chars[0] {
            'N' => parts.push("Noun"),
            'V' => parts.push("Verb"),
            'A' => parts.push("Adjective"),
            'R' => parts.push("Pronoun"),
            'D' => parts.push("Adverb"),
            'P' => parts.push("Preposition"),
            'C' => parts.push("Conjunction"),
            'I' => parts.push("Interjection"),
            _ => {}
        }

        // This is a simplified logic - Robinson's codes are positional.
        // Example: N-DSM -> Noun, Dative, Singular, Masculine
        for c in chars.iter().skip(1) {
            match c {
                '-' => continue,
                'N' => parts.push("Nominative"),
                'G' => parts.push("Genitive"),
                'D' => parts.push("Dative"),
                'A' => parts.push("Accusative"),
                'S' => parts.push("Singular"),
                'P' => parts.push("Plural"),
                'M' => parts.push("Masculine"),
                'F' => parts.push("Feminine"),
                'N' if parts.contains(&"Noun") => parts.push("Neuter"),
                _ => {}
            }
        }

        parts.join(", ")
    }

    unsafe fn sword_ptr_to_string(&self, ptr: *const std::os::raw::c_char) -> Option<String> {
        if ptr.is_null() {
            return None;
        }
        Some(unsafe { std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned() })
    }
}
