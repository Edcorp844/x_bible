use adw::prelude::*;
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
        adw::NavigationPage{
            #[wrap(Some)]
            set_child=&gtk::Box {
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
        }}
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
        let overlay = widgets.page_overlay.clone();

        let obox = gtk::Box::builder()
            .vexpand(true)
            .hexpand(true)
            .valign(gtk::Align::Start)
            .build();

        overlay.add_overlay(&obox);

        sender.input(StudyInput::LoadReference(query));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            StudyInput::LoadReference(refe) => {
                self.load_reference(&refe);
            }
            StudyInput::SelectStrong(_id) => {
                // Handle lexicon lookup selection here
            }
            StudyInput::SetModule(name) => {
                self.module = name;
            }
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
            // --- Stable CStrings (never inline) ---
            let key_ref = CString::new(reference).unwrap();

            let module_name = CString::new(self.module.as_str()).unwrap();
            let strong_opt = CString::new("Strong's Numbers").unwrap();
            let lemma_opt = CString::new("Lemmas").unwrap();
            let woc_opt = CString::new("Words of Christ").unwrap();
            let on = CString::new("On").unwrap();

            // 1. Load module
            let h_mod =
                org_crosswire_sword_SWMgr_getModuleByName(self.mgr_ptr, module_name.as_ptr());
            if h_mod == 0 {
                return verses;
            }

            org_crosswire_sword_SWMgr_setGlobalOption(
                self.mgr_ptr,
                strong_opt.as_ptr(),
                on.as_ptr(),
            );

            org_crosswire_sword_SWMgr_setGlobalOption(self.mgr_ptr, woc_opt.as_ptr(), on.as_ptr());

            org_crosswire_sword_SWMgr_setGlobalOption(
                self.mgr_ptr,
                lemma_opt.as_ptr(),
                on.as_ptr(),
            );

            // 3. Set reference
            org_crosswire_sword_SWModule_setKeyText(h_mod, key_ref.as_ptr());

            // 4. Read initial key (COPY IMMEDIATELY)
            let initial_key =
                match self.sword_ptr_to_string(org_crosswire_sword_SWModule_getKeyText(h_mod)) {
                    Some(k) => k,
                    None => return verses,
                };

            let chapter_boundary = initial_key
                .split(|c| c == ':' || c == '.')
                .next()
                .unwrap_or(&initial_key)
                .to_string();

            // 5. Iterate verses
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

                let number = self.extract_verse_number(&key);

                let html = match self
                    .sword_ptr_to_string(org_crosswire_sword_SWModule_renderText(h_mod))
                {
                    Some(h) => h,
                    None => break,
                };

                // println!("HTML: {}", html);
                let words = self.parse_single_verse_html(&html);

                verses.push(Verse {
                    osis_id: key,
                    number,
                    words,
                    notes: vec![],
                    is_paragraph_start: html.contains('¶') || number == 1,
                });

                // Move forward
                org_crosswire_sword_SWModule_next(h_mod);

                if org_crosswire_sword_SWModule_popError(h_mod) != 0 {
                    break;
                }
            }
        }

        verses
    }

    fn parse_single_verse_html(&self, html: &str) -> Vec<Word> {
        use ego_tree::NodeRef;
        use scraper::{Html, node::Node};

        let fragment = Html::parse_fragment(html);
        let mut words = Vec::new();

        fn walk(
            node: NodeRef<Node>,
            words: &mut Vec<Word>,
            current_strong: Option<String>,
            is_added: bool,
            is_italic: bool,
            is_bold_text: bool,
            in_jesus_block: bool, // New parameter to track state
        ) {
            match node.value() {
                Node::Text(t) => {
                    let text = t.text.trim();
                    if text.is_empty() {
                        return;
                    }

                    // 1. Strong's Number Handling (Same as your working version)
                    let stripped = text.trim_matches(|c: char| !c.is_ascii_alphanumeric());
                    if !stripped.is_empty() && stripped.chars().all(|c| c.is_ascii_digit()) {
                        if let Some(last) = words.last_mut() {
                            let lex = last.lex.get_or_insert_with(Default::default);
                            let prefix = if stripped.starts_with('0') || stripped.len() > 3 {
                                "H"
                            } else {
                                "G"
                            };
                            let id = format!("{}{}", prefix, stripped);
                            if !lex.strongs.contains(&id) {
                                lex.strongs.push(id);
                            }
                        } else if !stripped.is_empty() && stripped.chars().all(|c| !c.is_ascii()) {
                            if let Some(last) = words.last_mut() {
                                let lex = last.lex.get_or_insert_with(Default::default);
                                lex.lemma = Some(stripped.to_string());
                            }
                        }
                        return;
                    }

                    // 2. Word Creation
                    for piece in text.split_whitespace() {
                        let cleaned = piece
                            .trim_matches(|c: char| c == '<' || c == '>' || c == '`')
                            .trim();
                        if cleaned.is_empty() {
                            continue;
                        }

                        words.push(Word {
                            text: cleaned.to_string(),
                            style: if is_added {
                                SegmentStyle::Added
                            } else {
                                SegmentStyle::Plain
                            },
                            is_red: in_jesus_block,
                            is_italic: is_italic,
                            is_bold_text,
                            lex: current_strong.as_ref().map(|s| LexicalInfo {
                                strongs: vec![s.clone()],
                                ..Default::default()
                            }),
                            is_punctuation: cleaned.chars().all(|c| !c.is_alphanumeric()),
                            ..Default::default()
                        });
                    }
                }

                Node::Element(el) => {
                    let mut strong = current_strong.clone();
                    let mut jesus_state = in_jesus_block;
                    let mut is_italic = is_italic;
                    let mut is_bold = is_bold_text;
                    let mut added = is_added;

                    // --- Check for Red Letter Class ---
                    if el.name() == "span" {
                        if let Some(class_val) = el.attr("class") {
                            if class_val.contains("wordsOfJesus") {
                                jesus_state = true;
                            } else if class_val.contains("transChange-added") {
                                added = true;
                            }
                        }
                    }

                    if el.name() == "i" {
                        is_italic = true
                    }

                    if el.name() == "b" {
                        is_bold = true
                    }

                    // --- Keep your existing Strong's Link logic ---
                    if el.name() == "a" {
                        if let Some(href) = el.attr("href") {
                            if let Some(id) = href
                                .split("showStrong=")
                                .last()
                                .and_then(|s| s.split('#').next())
                            {
                                let prefix = if id.starts_with('0') { "H" } else { "G" };
                                strong = Some(format!("{}{}", prefix, id));
                            }
                        }
                    }

                    // Pass the potentially updated jesus_state to children
                    for child in node.children() {
                        walk(
                            child,
                            words,
                            strong.clone(),
                            added,
                            is_italic,
                            is_bold,
                            jesus_state,
                        );
                    }
                }
                _ => {}
            }
        }

        for child in fragment.tree.root().children() {
            walk(child, &mut words, None, false, false, false, false);
        }

        words
    }

    fn extract_verse_number(&self, key: &str) -> i32 {
        key.split(|c| c == '.' || c == ':')
            .last()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    unsafe fn sword_ptr_to_string(&self, ptr: *const c_char) -> Option<String> {
        if ptr.is_null() {
            return None;
        }

        // Copy immediately — never keep Sword pointers alive
        Some(unsafe { CStr::from_ptr(ptr).to_string_lossy().into_owned() })
    }
}
