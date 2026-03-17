use crate::features::{
    bible::components::page::helpers::TitleStyle,
    core::module_engine::sword_engine_module_content_ext::{
        LexicalInfo, Section, TextDirection, Verse, Word,
    },
};
use ego_tree::NodeRef;
use scraper::Html;

pub struct OsisTransilationEngine {}

impl OsisTransilationEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse_osis_to_sections(
        &self,
        language: String,
        osis: &str,
        verse_key: Option<String>,
    ) -> Vec<Section> {
        let (mut words, notes, title_words, title_style) = self.parse_osis_content(language, osis);

        // Detect direction based on the first few words
        let sample_text = words.first().map(|w| w.text.as_str()).unwrap_or("");
        let text_direction = self.detect_direction(sample_text);

        self.apply_group_metadata(&mut words);
        let mut title_vec = title_words.unwrap_or_default();
        self.apply_group_metadata(&mut title_vec);

        let key = verse_key.unwrap_or_default();
        let is_para =
            osis.contains("type=\"paragraph\"") || (!key.is_empty() && key.ends_with(":1"));

        vec![Section {
            title: title_vec,
            verses: vec![Verse {
                number: self.extract_verse_number(&key),
                osis_id: key,
                words,
                notes,
                is_paragraph_start: is_para,
            }],
            text_direction,
        }]
    }

    fn parse_osis_content(
        &self,
        language: String,
        osis: &str,
    ) -> (Vec<Word>, Vec<String>, Option<Vec<Word>>, TitleStyle) {
        let fragment = Html::parse_fragment(osis);
        let mut words = Vec::new();
        let mut verse_notes = Vec::new();
        let mut title_words = Vec::new();
        let mut title_style = TitleStyle::H3;

        self.walk_osis(
            fragment.tree.root(),
            &mut words,
            &mut verse_notes,
            &mut title_words,
            &mut title_style,
            None,
            false,
            false,
            false,
            false,
            false,
            false,
            language,
        );

        let final_title = if title_words.is_empty() {
            None
        } else {
            Some(title_words)
        };
        (words, verse_notes, final_title, title_style)
    }

    fn walk_osis(
        &self,
        node: NodeRef<scraper::node::Node>,
        words: &mut Vec<Word>,
        verse_notes: &mut Vec<String>,
        title_accumulator: &mut Vec<Word>,
        current_title_style: &mut TitleStyle,
        parent_lex: Option<LexicalInfo>,
        is_red: bool,
        is_added: bool,
        is_italic: bool,
        is_inside_title: bool,
        is_inside_note: bool,
        is_divine: bool,
        language: String,
    ) {
        use scraper::node::Node;

        match node.value() {
            Node::Element(el) => {
                let mut active_lex = parent_lex.clone();
                let mut active_red = is_red;
                let mut active_added = is_added;
                let mut active_italic = is_italic;
                let mut active_divine = is_divine;
                let mut traversing_title = is_inside_title;
                let mut traversing_note = is_inside_note;

                match el.name() {
                    "title" => {
                        traversing_title = true;
                        let level = el
                            .attr("level")
                            .and_then(|l| l.parse::<u8>().ok())
                            .unwrap_or(3);
                        *current_title_style = match level {
                            1 => TitleStyle::H1,
                            2 => TitleStyle::H2,
                            _ => TitleStyle::H3,
                        };
                    }
                    "w" => {
                        let raw_lemma = el.attr("lemma").unwrap_or("");
                        active_lex = Some(LexicalInfo {
                            strongs: raw_lemma
                                .split_whitespace()
                                .filter(|s| s.starts_with("strong:"))
                                .map(|s| s.trim_start_matches("strong:").to_string())
                                .collect(),
                            ..Default::default()
                        });
                    }
                    "divineName" => active_divine = true,
                    "q" if el.attr("who") == Some("Jesus") => active_red = true,
                    "transChange" if el.attr("type") == Some("added") => active_added = true,
                    "hi" if el.attr("type") == Some("italic") => active_italic = true,
                    "note" => {
                        traversing_note = true;
                        let text = self.collect_note_text(node);
                        if !text.is_empty() {
                            verse_notes.push(text);
                        }
                        return;
                    }
                    _ => {}
                }

                for child in node.children() {
                    self.walk_osis(
                        child,
                        words,
                        verse_notes,
                        title_accumulator,
                        current_title_style,
                        active_lex.clone(),
                        active_red,
                        active_added,
                        active_italic,
                        traversing_title,
                        traversing_note,
                        active_divine,
                        language.clone(),
                    );
                }
            }
            Node::Text(t) => {
                if is_inside_note {
                    return;
                }
                let text = t.text.trim();
                if text.is_empty() {
                    return;
                }

                if text.contains('<') && (text.contains("<w") || text.contains("</w>")) {
                    let sub_fragment = Html::parse_fragment(text);
                    for child in sub_fragment.tree.root().children() {
                        self.walk_osis(
                            child,
                            words,
                            verse_notes,
                            title_accumulator,
                            current_title_style,
                            parent_lex.clone(),
                            is_red,
                            is_added,
                            is_italic,
                            is_inside_title,
                            is_inside_note,
                            is_divine,
                            language.clone(),
                        );
                    }
                } else {
                    let target_vec = if is_inside_title {
                        title_accumulator
                    } else {
                        words
                    };

                    // --- JAPANESE / CHINESE FIX ---
                    if self.is_non_segmented(text) {
                        for c in text.chars() {
                            if c.is_whitespace() {
                                continue;
                            }
                            target_vec.push(self.create_word(
                                c.to_string(),
                                is_added,
                                is_red,
                                is_italic,
                                is_inside_title,
                                is_divine,
                                parent_lex.clone(),
                                language.clone(),
                            ));
                        }
                    } else {
                        // STANDARD WHITESPACE SPLIT (Azerbaijani, English, etc.)
                        for piece in text.split_whitespace() {
                            target_vec.push(self.create_word(
                                piece.to_string(),
                                is_added,
                                is_red,
                                is_italic,
                                is_inside_title,
                                is_divine,
                                parent_lex.clone(),
                                language.clone(),
                            ));
                        }
                    }
                }
            }
            _ => {
                for child in node.children() {
                    self.walk_osis(
                        child,
                        words,
                        verse_notes,
                        title_accumulator,
                        current_title_style,
                        parent_lex.clone(),
                        is_red,
                        is_added,
                        is_italic,
                        is_inside_title,
                        is_inside_note,
                        is_divine,
                        language.clone(),
                    );
                }
            }
        }
    }

    // Helper to create Word struct to keep walk_osis clean
    fn create_word(
        &self,
        text: String,
        is_added: bool,
        is_red: bool,
        is_italic: bool,
        is_inside_title: bool,
        is_divine: bool,
        lex: Option<LexicalInfo>,
        language: String,
    ) -> Word {
        let is_punct = text
            .chars()
            .all(|c| c.is_ascii_punctuation() || ('\u{3000}'..='\u{303F}').contains(&c));
        Word {
            text,
            is_red,
            is_italic,
            is_bold_text: is_inside_title || is_divine,
            lex,
            note: None,
            is_first_in_group: false,
            is_last_in_group: false,
            is_title: is_inside_title,
            is_punctuation: is_punct,
            language,
        }
    }

    fn is_non_segmented(&self, text: &str) -> bool {
        text.chars().any(|c| {
            ('\u{4E00}'..='\u{9FFF}').contains(&c) || // CJK Ideographs
            ('\u{3040}'..='\u{30FF}').contains(&c) // Hiragana/Katakana
        })
    }

    fn collect_note_text(&self, node: NodeRef<scraper::node::Node>) -> String {
        node.descendants()
            .filter_map(|n| n.value().as_text())
            .map(|t| t.text.as_ref())
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }

    fn apply_group_metadata(&self, words: &mut [Word]) {
        let len = words.len();
        if len == 0 {
            return;
        }
        for i in 0..len {
            let current_is_red = words[i].is_red;

            if current_is_red {
                let prev = i > 0 && (current_is_red && words[i - 1].is_red);
                let next = i < len - 1 && (current_is_red && words[i + 1].is_red);
                words[i].is_first_in_group = !prev;
                words[i].is_last_in_group = !next;
            }
        }
    }

    fn extract_verse_number(&self, key: &str) -> i32 {
        key.split(|c| c == '.' || c == ':')
            .last()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }

    fn detect_direction(&self, text: &str) -> TextDirection {
        let is_rtl = text.chars().any(|c| {
            ('\u{0600}'..='\u{06FF}').contains(&c) || // Arabic
            ('\u{0750}'..='\u{077F}').contains(&c) || // Arabic Ext
            ('\u{0590}'..='\u{05FF}').contains(&c) // Hebrew
        });
        if is_rtl {
            TextDirection::Rtl
        } else {
            TextDirection::Ltr
        }
    }
}
