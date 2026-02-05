use crate::features::bible::transilation_engines::osis_engine::helpers::{
    BibleVersion, Book, Chapter, LexicalInfo, SegmentStyle, Verse, Word,
};
use quick_xml::{Reader, events::Event};

pub struct OsisEngine;

impl OsisEngine {
    /// Targeted parsing for a specific chapter (e.g., "Gen.1").
    pub fn parse_verses<R: std::io::BufRead>(
        reader: &mut Reader<R>,
        target_chapter: &str,
    ) -> Vec<Verse> {
        let mut verses = Vec::new();
        let mut buf = Vec::new();

        let mut current_verse: Option<Verse> = None;
        let mut style_stack = vec![SegmentStyle::Plain];
        let mut current_lex: Option<LexicalInfo> = None;

        let mut inside_target_chapter = false;
        let mut inside_paragraph = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let name = e.name();
                    match name.as_ref() {
                        b"chapter" => {
                            let osis_id = Self::get_attr(reader, e, b"osisID").unwrap_or_default();
                            if inside_target_chapter
                                && !osis_id.is_empty()
                                && osis_id != target_chapter
                            {
                                break;
                            }
                            if osis_id == target_chapter {
                                inside_target_chapter = true;
                            }
                        }
                        b"p" => inside_paragraph = true,
                        b"verse" if inside_target_chapter => {
                            let sid = Self::get_attr(reader, e, b"sID");
                            let eid = Self::get_attr(reader, e, b"eID");
                            let osis = Self::get_attr(reader, e, b"osisID");

                            if let Some(id) = sid.or(osis.clone()) {
                                let number = Self::get_attr(reader, e, b"n")
                                    .unwrap_or_else(|| Self::verse_number_from_osis(&id));
                                current_verse = Some(Verse {
                                    osis_id: id,
                                    number: number,
                                    words: Vec::new(),
                                    notes: Vec::new(),
                                    is_paragraph_start: inside_paragraph,
                                });
                                inside_paragraph = false;
                            } else if eid.is_some() {
                                if let Some(v) = current_verse.take() {
                                    verses.push(v);
                                }
                            }
                        }
                        b"q" if inside_target_chapter => {
                            if Self::get_attr(reader, e, b"who").as_deref() == Some("Jesus") {
                                style_stack.push(SegmentStyle::RedLetter);
                            }
                        }
                        b"transChange" if inside_target_chapter => {
                            style_stack.push(SegmentStyle::Added);
                        }
                        b"note" if inside_target_chapter => {
                            style_stack.push(SegmentStyle::Note);
                        }
                        b"w" if inside_target_chapter => {
                            current_lex = Some(LexicalInfo {
                                strongs: Self::parse_strongs(Self::get_attr(reader, e, b"lemma")),
                                morph: Self::parse_morph(Self::get_attr(reader, e, b"morph")),
                                lemma: None,
                            });
                        }
                        _ => {}
                    }
                }
                Ok(Event::Text(e)) if inside_target_chapter => {
                    if let Some(v) = current_verse.as_mut() {
                        let text = reader.decoder().decode(e.as_ref()).unwrap_or_default();
                        if style_stack.contains(&SegmentStyle::Note) {
                            v.notes.push(text.clone().to_string());
                        } else {
                            Self::process_text_into_words(v, &text, &style_stack, &current_lex);
                            current_lex = None;
                        }
                    }
                }
                Ok(Event::End(ref e)) if inside_target_chapter => match e.name().as_ref() {
                    b"q" | b"transChange" | b"note" => {
                        style_stack.pop();
                    }
                    _ => {}
                },
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }
        verses
    }

    /// Scans metadata and structure for UI selection.
    pub fn parse_books<R: std::io::BufRead>(reader: &mut Reader<R>) -> Vec<Book> {
        let mut books = Vec::new();
        let mut buf = Vec::new();
        let mut current_book: Option<Book> = None;
        let mut current_tag: Option<String> = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    let name = reader
                        .decoder()
                        .decode(e.name().as_ref())
                        .unwrap_or_default()
                        .into_owned();
                    match name.as_str() {
                        "div" if Self::get_attr(reader, e, b"type").as_deref() == Some("book") => {
                            if let Some(b) = current_book.take() {
                                books.push(b);
                            }
                            current_book = Some(Book {
                                osis_id: Self::get_attr(reader, e, b"osisID").unwrap_or_default(),
                                title: String::new(),
                                chapters: Vec::new(),
                                canonical: true,
                            });
                        }
                        "chapter" => {
                            if let Some(b) = current_book.as_mut() {
                                b.chapters.push(Chapter {
                                    title: Self::get_attr(reader, e, b"chapterTitle")
                                        .unwrap_or_default(),
                                    osis_ref: Self::get_attr(reader, e, b"osisID")
                                        .unwrap_or_default(),
                                    number: Self::get_attr(reader, e, b"n").unwrap_or_default(),
                                    verses: Vec::new(),
                                });
                            }
                        }
                        "title" => current_tag = Some("title".to_string()),
                        _ => {}
                    }
                }
                Ok(Event::Text(e)) => {
                    if let (Some(tag), Some(book)) = (current_tag.as_ref(), current_book.as_mut()) {
                        if tag == "title" && book.title.is_empty() {
                            // FIX: Consistent decoder use
                            book.title = reader
                                .decoder()
                                .decode(e.as_ref())
                                .unwrap_or_default()
                                .into_owned();
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    if e.name().as_ref() == b"title" {
                        current_tag = None;
                    }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }
        if let Some(b) = current_book {
            books.push(b);
        }
        books
    }

    pub fn parse_version<R: std::io::BufRead>(reader: &mut Reader<R>) -> Option<BibleVersion> {
        let mut buf = Vec::new();

        let mut in_header = false;
        let mut in_work = false;
        let mut is_bible_work = false;

        let mut current_tag: Option<String> = None;
        let mut version: Option<BibleVersion> = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"header" => {
                    in_header = true;
                }

                Ok(Event::End(ref e)) if e.name().as_ref() == b"header" => {
                    break;
                }

                Ok(Event::Start(ref e)) if in_header && e.name().as_ref() == b"work" => {
                    in_work = true;
                    is_bible_work = false;
                    current_tag = None;

                    let osis_id = Self::get_attr(reader, e, b"osisWork").unwrap_or_default();

                    version = Some(BibleVersion {
                        osis_id,
                        title: String::new(),
                        identifier: String::new(),
                        scope: String::new(),
                        ref_system: String::new(),
                    });
                }

                Ok(Event::Start(ref e)) if in_header && in_work => {
                    let name = reader
                        .decoder()
                        .decode(e.name().as_ref())
                        .unwrap_or_default()
                        .into_owned();

                    match name.as_str() {
                        "title" | "identifier" | "scope" | "refSystem" => current_tag = Some(name),
                        _ => {}
                    }
                }

                Ok(Event::Text(e)) if in_header && in_work => {
                    if let (Some(tag), Some(ver)) = (current_tag.as_ref(), version.as_mut()) {
                        let text = reader
                            .decoder()
                            .decode(e.as_ref())
                            .unwrap_or_default()
                            .trim()
                            .to_string();

                        if text.is_empty() {
                            continue;
                        }

                        match tag.as_str() {
                            "title" => ver.title.push_str(&text),
                            "identifier" => ver.identifier.push_str(&text),
                            "scope" => ver.scope.push_str(&text),
                            "refSystem" => {
                                ver.ref_system.push_str(&text);
                                if text.starts_with("Bible.") {
                                    is_bible_work = true;
                                }
                            }
                            _ => {}
                        }
                    }
                }

                Ok(Event::End(ref e)) if e.name().as_ref() == b"work" => {
                    if is_bible_work {
                        return version;
                    }

                    in_work = false;
                    current_tag = None;
                    version = None;
                }

                Ok(Event::Eof) => break,
                _ => {}
            }

            buf.clear();
        }

        None
    }

    fn get_attr<R: std::io::BufRead>(
        reader: &Reader<R>,
        e: &quick_xml::events::BytesStart,
        name: &[u8],
    ) -> Option<String> {
        e.attributes()
            .flatten()
            .find(|a| a.key.as_ref() == name)
            .map(|a| {
                reader
                    .decoder()
                    .decode(&a.value)
                    .unwrap_or_default()
                    .into_owned()
            })
    }

    fn process_text_into_words(
        v: &mut Verse,
        raw_text: &str,
        styles: &[SegmentStyle],
        lex: &Option<LexicalInfo>,
    ) {
        let style = *styles.last().unwrap_or(&SegmentStyle::Plain);
        let is_red = styles.contains(&SegmentStyle::RedLetter);

        for part in raw_text.split_whitespace() {
            let mut word_str = part.to_string();
            let mut punc = String::new();

            while let Some(c) = word_str.chars().last() {
                if c.is_ascii_punctuation() && c != '\'' {
                    punc.insert(0, word_str.pop().unwrap());
                } else {
                    break;
                }
            }

            v.words.push(Word {
                text: word_str,
                style,
                is_red,
                lex: lex.clone(),
                is_first_in_group: false,
                is_last_in_group: false,
                is_punctuation: false,
            });

            if !punc.is_empty() {
                v.words.push(Word {
                    text: punc,
                    style: SegmentStyle::Plain,
                    is_red,
                    lex: None,
                    is_first_in_group: false,
                    is_last_in_group: false,
                    is_punctuation: true,
                });
            }
        }
    }

    fn parse_strongs(attr: Option<String>) -> Vec<String> {
        attr.unwrap_or_default()
            .split_whitespace()
            .filter_map(|s| s.strip_prefix("strong:"))
            .map(|s| s.to_string())
            .collect()
    }

    fn parse_morph(attr: Option<String>) -> Option<String> {
        attr.and_then(|s| s.strip_prefix("strongMorph:").map(|m| m.to_string()))
    }

    fn verse_number_from_osis(osis_id: &str) -> String {
        osis_id.rsplit('.').next().unwrap_or_default().to_string()
    }
}
