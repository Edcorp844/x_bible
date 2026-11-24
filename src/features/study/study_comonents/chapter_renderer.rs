use std::{collections::BTreeMap, rc::Rc};

use crate::core::ui::{
    highlights::{HighlightColor, HighlightStore, HighlightStyle},
    text_config::{AddedWordsStyle, TextConfig, VerseLayout},
};
use adw::prelude::*;
use gettextrs::gettext;
use gtk::pango;
use xbible_engine::{
    features::reader::{EntityRef, ReaderBlock, ReaderChapter, ReaderVerse},
    sources::models::GenericTextSpan,
};

#[derive(Debug, Clone)]
pub(crate) enum StudyToolKind {
    Dictionary,
    Lexicon,
    Commentary,
}

#[derive(Debug, Clone)]
pub(crate) enum VerseContextAction {
    Highlight(HighlightColor),
    ClearHighlight,
    Study(StudyToolKind),
    Copy,
    Share,
}

#[derive(Debug, Clone)]
pub(crate) enum ChapterViewEvent {
    VerseAction {
        book_id: String,
        chapter: i32,
        verse: i32,
        action: VerseContextAction,
    },
    EntitySelected(EntityRef),
}

struct ParagraphSection {
    heading: Option<String>,
    subtitle: Option<String>,
    verses: Vec<ReaderVerse>,
}

pub(crate) fn build_chapter_view(
    chapter: &ReaderChapter,
    config: &TextConfig,
    highlights: &HighlightStore,
    entities: &[EntityRef],
    selected_verse: Option<i32>,
    on_event: impl Fn(ChapterViewEvent) + 'static,
) -> gtk::Box {
    let callback: Rc<dyn Fn(ChapterViewEvent)> = Rc::new(on_event);
    let base_pt = 17.0 * (config.font_size_percent.clamp(50, 300) as f64 / 100.0);
    let line_spacing_pt = (base_pt * config.line_spacing.max(0.8)) - base_pt;
    let verse_spacing_px = ((base_pt * config.verse_spacing.max(0.8)) - base_pt).max(4.0) as i32;

    let root = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(0)
        .hexpand(true)
        .halign(gtk::Align::Fill)
        .margin_top(16)
        .margin_bottom(32)
        .margin_start(20)
        .margin_end(20)
        .build();

    // Chapter header
    let header_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(8)
        .halign(gtk::Align::Fill)
        .margin_top(8)
        .margin_bottom(24)
        .build();

    let book_title = gtk::Label::builder()
        .label(&chapter.book_name.to_uppercase())
        .halign(gtk::Align::Center)
        .wrap(true)
        .build();
    book_title.set_attributes(Some(&make_attr_list(
        config,
        base_pt * 0.9,
        pango::Weight::Medium,
        None,
        None,
        0.0,
    )));

    let chapter_row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Fill)
        .valign(gtk::Align::Center)
        .build();

    let left_rule = gtk::Separator::new(gtk::Orientation::Horizontal);
    left_rule.set_hexpand(true);
    left_rule.set_valign(gtk::Align::Center);

    let chapter_num = gtk::Label::builder()
        .label(&chapter.chapter_number.to_string())
        .halign(gtk::Align::Center)
        .build();
    chapter_num.add_css_class("accent");
    chapter_num.set_attributes(Some(&make_attr_list(
        config,
        base_pt * 2.2,
        pango::Weight::Bold,
        None,
        None,
        0.0,
    )));

    let right_rule = gtk::Separator::new(gtk::Orientation::Horizontal);
    right_rule.set_hexpand(true);
    right_rule.set_valign(gtk::Align::Center);

    chapter_row.append(&left_rule);
    chapter_row.append(&chapter_num);
    chapter_row.append(&right_rule);

    header_box.append(&book_title);
    header_box.append(&chapter_row);
    root.append(&header_box);

    // Sections
    let sections = group_blocks_into_sections(&chapter.blocks);
    for section in sections {
        let sec_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(4)
            .halign(gtk::Align::Fill)
            .margin_bottom(16)
            .build();

        if let Some(ref heading) = section.heading {
            let h_lbl = gtk::Label::builder()
                .label(heading)
                .halign(gtk::Align::Center)
                .justify(gtk::Justification::Center)
                .wrap(true)
                .margin_top(16)
                .margin_bottom(if section.subtitle.is_none() { 10 } else { 4 })
                .build();
            h_lbl.set_attributes(Some(&make_attr_list(
                config,
                base_pt * 1.025,
                pango::Weight::Semibold,
                None,
                None,
                line_spacing_pt,
            )));
            sec_box.append(&h_lbl);
        }

        if let Some(ref subtitle) = section.subtitle {
            let s_lbl = gtk::Label::builder()
                .label(subtitle)
                .halign(gtk::Align::Center)
                .justify(gtk::Justification::Center)
                .wrap(true)
                .margin_top(if section.heading.is_none() { 10 } else { 0 })
                .margin_bottom(12)
                .build();
            s_lbl.add_css_class("dim-label");
            let attrs = make_attr_list(
                config,
                base_pt * 0.85,
                pango::Weight::Normal,
                None,
                None,
                line_spacing_pt * 0.8,
            );
            let italic = pango::AttrInt::new_style(pango::Style::Italic);
            attrs.insert(italic);
            s_lbl.set_attributes(Some(&attrs));
            sec_box.append(&s_lbl);
        }

        if !section.verses.is_empty() {
            match config.verse_layout {
                VerseLayout::VersePerLine => {
                    let list_box = gtk::Box::builder()
                        .orientation(gtk::Orientation::Vertical)
                        .spacing(verse_spacing_px)
                        .halign(gtk::Align::Fill)
                        .build();

                    for verse in &section.verses {
                        let hl = highlights.get(
                            &chapter.book_id,
                            chapter.chapter_number as u32,
                            verse.number as u32,
                        );
                        let is_selected = selected_verse == Some(verse.number);
                        let row = build_verse_line_row(
                            &chapter.book_id,
                            chapter.chapter_number,
                            verse,
                            config,
                            base_pt,
                            line_spacing_pt,
                            hl.map(|h| (h.color, h.style)),
                            is_selected,
                            Rc::clone(&callback),
                        );
                        list_box.append(&row);
                    }
                    sec_box.append(&list_box);
                }
                VerseLayout::Paragraph => {
                    let target_verse = section.verses.first().map(|v| v.number).unwrap_or(1);
                    let para_box = build_paragraph_block(
                        &chapter.book_id,
                        chapter.chapter_number,
                        target_verse,
                        &section.verses,
                        config,
                        highlights,
                        base_pt,
                        line_spacing_pt,
                        selected_verse,
                        Rc::clone(&callback),
                    );
                    sec_box.append(&para_box);
                }
            }
        }

        root.append(&sec_box);
    }

    if !entities.is_empty() {
        root.append(&build_chapter_footer(entities, Rc::clone(&callback)));
    }

    root
}

#[allow(clippy::too_many_arguments)]
fn build_verse_line_row(
    book_id: &str,
    chapter_num: i32,
    verse: &ReaderVerse,
    config: &TextConfig,
    base_pt: f64,
    line_spacing_pt: f64,
    highlight: Option<(HighlightColor, HighlightStyle)>,
    is_selected: bool,
    callback: Rc<dyn Fn(ChapterViewEvent)>,
) -> gtk::Box {
    let row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .halign(gtk::Align::Fill)
        .build();

    if config.show_verse_numbers && verse.number > 0 {
        let num_text = if config.show_verse_notes && !verse.footnote_ids.is_empty() {
            format!("{}*", verse.number)
        } else {
            verse.number.to_string()
        };
        let num_lbl = gtk::Label::builder()
            .label(&num_text)
            .valign(gtk::Align::Start)
            .build();
        num_lbl.add_css_class("accent");
        num_lbl.set_attributes(Some(&make_attr_list(
            config,
            base_pt * 0.5,
            pango::Weight::Bold,
            None,
            None,
            0.0,
        )));
        row.append(&num_lbl);
    }

    let text = render_spans_text(&verse.spans, config, Some(config.word_spacing));
    let body_lbl = gtk::Label::builder()
        .label(&text)
        .halign(gtk::Align::Fill)
        .hexpand(true)
        .wrap(true)
        .justify(if config.justify {
            gtk::Justification::Fill
        } else {
            gtk::Justification::Left
        })
        .build();

    let bg_hex = if is_selected {
        Some("#3584E44D")
    } else if let Some((color, style)) = highlight
        && matches!(style, HighlightStyle::Solid)
    {
        Some(color.rgba_hex())
    } else {
        None
    };

    let ul_hex = if let Some((color, style)) = highlight
        && matches!(
            style,
            HighlightStyle::Underline | HighlightStyle::DashedUnderline
        ) {
        Some(color.rgba_hex())
    } else {
        None
    };

    body_lbl.set_attributes(Some(&make_attr_list(
        config,
        base_pt,
        pango::Weight::Normal,
        bg_hex,
        ul_hex,
        line_spacing_pt,
    )));
    row.append(&body_lbl);

    attach_verse_context_menu(
        &row,
        book_id.to_string(),
        chapter_num,
        verse.number,
        callback,
    );

    row
}

#[allow(clippy::too_many_arguments)]
fn build_paragraph_block(
    book_id: &str,
    chapter_num: i32,
    action_verse_num: i32,
    verses: &[ReaderVerse],
    config: &TextConfig,
    highlights: &HighlightStore,
    base_pt: f64,
    line_spacing_pt: f64,
    selected_verse: Option<i32>,
    callback: Rc<dyn Fn(ChapterViewEvent)>,
) -> gtk::Box {
    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .halign(gtk::Align::Fill)
        .build();

    let mut combined = String::new();
    let mut number_ranges = Vec::new();
    for (idx, verse) in verses.iter().enumerate() {
        if idx > 0 {
            combined.push(' ');
        }
        if config.show_verse_numbers && verse.number > 0 {
            let start = combined.len() as u32;
            combined.push_str(&verse.number.to_string());
            if config.show_verse_notes && !verse.footnote_ids.is_empty() {
                combined.push('*');
            }
            let end = combined.len() as u32;
            number_ranges.push((start, end));
            combined.push(' ');
        }
        combined.push_str(&render_spans_text(
            &verse.spans,
            config,
            Some(config.word_spacing),
        ));
    }

    let first_hl = verses
        .first()
        .and_then(|v| highlights.get(book_id, chapter_num as u32, v.number as u32));
    let is_selected = selected_verse.is_some_and(|sv| verses.iter().any(|v| v.number == sv));

    let bg_hex = if is_selected {
        Some("#3584E44D")
    } else if let Some(hl) = first_hl
        && matches!(hl.style, HighlightStyle::Solid)
    {
        Some(hl.color.rgba_hex())
    } else {
        None
    };

    let ul_hex = if let Some(hl) = first_hl
        && matches!(
            hl.style,
            HighlightStyle::Underline | HighlightStyle::DashedUnderline
        ) {
        Some(hl.color.rgba_hex())
    } else {
        None
    };

    let label = gtk::Label::builder()
        .label(&combined)
        .halign(gtk::Align::Fill)
        .hexpand(true)
        .wrap(true)
        .justify(if config.justify {
            gtk::Justification::Fill
        } else {
            gtk::Justification::Left
        })
        .build();
    let attrs = make_attr_list(
        config,
        base_pt,
        pango::Weight::Normal,
        bg_hex,
        ul_hex,
        line_spacing_pt,
    );
    for (start, end) in number_ranges {
        let mut size = pango::AttrSize::new(((base_pt * 0.5 * pango::SCALE as f64) as i32).max(1));
        size.set_start_index(start);
        size.set_end_index(end);
        attrs.insert(size);
        let mut weight = pango::AttrInt::new_weight(pango::Weight::Bold);
        weight.set_start_index(start);
        weight.set_end_index(end);
        attrs.insert(weight);
        let mut color = pango::AttrColor::new_foreground(0x35 * 257, 0x84 * 257, 0xe4 * 257);
        color.set_start_index(start);
        color.set_end_index(end);
        attrs.insert(color);
    }
    label.set_attributes(Some(&attrs));

    container.append(&label);
    attach_verse_context_menu(
        &container,
        book_id.to_string(),
        chapter_num,
        action_verse_num,
        callback,
    );

    container
}

fn render_spans_text(
    spans: &[GenericTextSpan],
    config: &TextConfig,
    word_spacing_px: Option<f64>,
) -> String {
    let mut out = String::new();
    let extra_space = word_spacing_px
        .map(|w| if w > 12.0 { "  " } else { " " })
        .unwrap_or(" ");

    for (idx, span) in spans.iter().enumerate() {
        let formatted = match config.added_words_style {
            AddedWordsStyle::Brackets if span.formatting.is_italic => {
                format!("[{}]", span.text)
            }
            _ => span.text.clone(),
        };
        out.push_str(&formatted);

        if let Some(next) = spans.get(idx + 1)
            && should_space(&span.text, &next.text)
        {
            out.push_str(extra_space);
        }
    }

    out
}

fn should_space(prev: &str, next: &str) -> bool {
    let Some(last_ch) = prev.chars().next_back() else {
        return false;
    };
    let Some(first_ch) = next.chars().next() else {
        return false;
    };

    if last_ch.is_whitespace() || first_ch.is_whitespace() {
        return false;
    }

    let no_space_before = [',', '.', ';', ':', '!', '?', ')', ']', '}', '%'];
    if no_space_before.contains(&first_ch) {
        return false;
    }

    let no_space_after = ['(', '[', '{'];
    if no_space_after.contains(&last_ch) {
        return false;
    }

    if first_ch == '\'' && next.chars().count() > 1 {
        return false;
    }

    true
}

fn group_blocks_into_sections(blocks: &[ReaderBlock]) -> Vec<ParagraphSection> {
    let mut sections = Vec::new();
    let mut heading: Option<String> = None;
    let mut subtitle: Option<String> = None;
    let mut verses: Vec<ReaderVerse> = Vec::new();

    for block in blocks {
        match block.item_type.as_str() {
            "heading" => {
                if heading.is_some() || subtitle.is_some() || !verses.is_empty() {
                    sections.push(ParagraphSection {
                        heading: heading.take(),
                        subtitle: subtitle.take(),
                        verses: std::mem::take(&mut verses),
                    });
                }
                heading = Some(block.text.clone());
            }
            "hebrew_subtitle" => {
                if heading.is_some() || subtitle.is_some() || !verses.is_empty() {
                    sections.push(ParagraphSection {
                        heading: heading.take(),
                        subtitle: subtitle.take(),
                        verses: std::mem::take(&mut verses),
                    });
                }
                subtitle = Some(block.text.clone());
            }
            "verse" => {
                verses.extend(block.verses.iter().cloned());
            }
            _ => {}
        }
    }

    if heading.is_some() || subtitle.is_some() || !verses.is_empty() {
        sections.push(ParagraphSection {
            heading,
            subtitle,
            verses,
        });
    }

    sections
}

fn make_attr_list(
    config: &TextConfig,
    size_pt: f64,
    weight: pango::Weight,
    bg_hex: Option<&str>,
    ul_hex: Option<&str>,
    line_spacing_pt: f64,
) -> pango::AttrList {
    let attrs = pango::AttrList::new();
    let pango_size = ((size_pt.max(6.0)) * pango::SCALE as f64) as i32;
    attrs.insert(pango::AttrSize::new(pango_size));
    attrs.insert(pango::AttrInt::new_weight(weight));
    attrs.insert(pango::AttrString::new_family(config.resolved_font_family()));

    if line_spacing_pt > 0.0 {
        let rise_or_spacing = (line_spacing_pt * pango::SCALE as f64) as i32;
        attrs.insert(pango::AttrInt::new_rise(rise_or_spacing));
    }

    if let Some(hex) = bg_hex
        && let Ok(rgba) = gtk::gdk::RGBA::parse(hex)
    {
        let r = (rgba.red() * 65535.0) as u16;
        let g = (rgba.green() * 65535.0) as u16;
        let b = (rgba.blue() * 65535.0) as u16;
        attrs.insert(pango::AttrColor::new_background(r, g, b));
    }

    if let Some(hex) = ul_hex
        && let Ok(rgba) = gtk::gdk::RGBA::parse(hex)
    {
        attrs.insert(pango::AttrInt::new_underline(pango::Underline::Single));
        let r = (rgba.red() * 65535.0) as u16;
        let g = (rgba.green() * 65535.0) as u16;
        let b = (rgba.blue() * 65535.0) as u16;
        attrs.insert(pango::AttrColor::new_underline_color(r, g, b));
    }

    attrs
}

fn attach_verse_context_menu(
    widget: &impl IsA<gtk::Widget>,
    book_id: String,
    chapter_num: i32,
    verse_num: i32,
    callback: Rc<dyn Fn(ChapterViewEvent)>,
) {
    let popover = gtk::Popover::new();
    popover.set_has_arrow(true);

    let menu_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .build();

    let hl_title = gtk::Label::builder()
        .label(&gettext("Highlight"))
        .halign(gtk::Align::Start)
        .build();
    hl_title.add_css_class("heading");
    menu_box.append(&hl_title);

    let colors_row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .build();

    for color in HighlightColor::ALL {
        let btn = gtk::Button::builder().label(&color.display_name()).build();
        btn.add_css_class("flat");
        let pop_weak = popover.downgrade();
        let cb = Rc::clone(&callback);
        let b_id = book_id.clone();
        btn.connect_clicked(move |_| {
            cb(ChapterViewEvent::VerseAction {
                book_id: b_id.clone(),
                chapter: chapter_num,
                verse: verse_num,
                action: VerseContextAction::Highlight(color),
            });
            if let Some(pop) = pop_weak.upgrade() {
                pop.popdown();
            }
        });
        colors_row.append(&btn);
    }
    menu_box.append(&colors_row);

    let clear_btn = gtk::Button::builder()
        .label(&gettext("Clear Highlight"))
        .halign(gtk::Align::Start)
        .build();
    clear_btn.add_css_class("flat");
    clear_btn.add_css_class("destructive-action");
    {
        let pop_weak = popover.downgrade();
        let cb = Rc::clone(&callback);
        let b_id = book_id.clone();
        clear_btn.connect_clicked(move |_| {
            cb(ChapterViewEvent::VerseAction {
                book_id: b_id.clone(),
                chapter: chapter_num,
                verse: verse_num,
                action: VerseContextAction::ClearHighlight,
            });
            if let Some(pop) = pop_weak.upgrade() {
                pop.popdown();
            }
        });
    }
    menu_box.append(&clear_btn);
    menu_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    for (label, tool) in [
        (gettext("Commentary"), StudyToolKind::Commentary),
        (gettext("Lexicon"), StudyToolKind::Lexicon),
        (gettext("Dictionary"), StudyToolKind::Dictionary),
    ] {
        let btn = gtk::Button::builder()
            .label(&format!("{} {label}", gettext("Study:")))
            .halign(gtk::Align::Start)
            .build();
        btn.add_css_class("flat");
        let pop_weak = popover.downgrade();
        let cb = Rc::clone(&callback);
        let b_id = book_id.clone();
        btn.connect_clicked(move |_| {
            cb(ChapterViewEvent::VerseAction {
                book_id: b_id.clone(),
                chapter: chapter_num,
                verse: verse_num,
                action: VerseContextAction::Study(tool.clone()),
            });
            if let Some(pop) = pop_weak.upgrade() {
                pop.popdown();
            }
        });
        menu_box.append(&btn);
    }

    menu_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    for (label, action) in [
        (gettext("Copy Verse"), VerseContextAction::Copy),
        (gettext("Share"), VerseContextAction::Share),
    ] {
        let btn = gtk::Button::builder()
            .label(&label)
            .halign(gtk::Align::Start)
            .build();
        btn.add_css_class("flat");
        let pop_weak = popover.downgrade();
        let cb = Rc::clone(&callback);
        let b_id = book_id.clone();
        let act = action.clone();
        btn.connect_clicked(move |_| {
            cb(ChapterViewEvent::VerseAction {
                book_id: b_id.clone(),
                chapter: chapter_num,
                verse: verse_num,
                action: act.clone(),
            });
            if let Some(pop) = pop_weak.upgrade() {
                pop.popdown();
            }
        });
        menu_box.append(&btn);
    }

    popover.set_child(Some(&menu_box));
    popover.set_parent(widget);

    let gesture = gtk::GestureClick::new();
    gesture.set_button(3);
    let pop_weak = popover.downgrade();
    gesture.connect_released(move |_, _, x, y| {
        if let Some(pop) = pop_weak.upgrade() {
            pop.set_pointing_to(Some(&gtk::gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
            pop.popup();
        }
    });
    widget.add_controller(gesture);
}

fn build_chapter_footer(
    entities: &[EntityRef],
    callback: Rc<dyn Fn(ChapterViewEvent)>,
) -> gtk::Box {
    let footer = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .halign(gtk::Align::Fill)
        .margin_top(32)
        .margin_bottom(48)
        .build();

    let header_row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(12)
        .halign(gtk::Align::Fill)
        .valign(gtk::Align::Center)
        .build();

    let left_sep = gtk::Separator::new(gtk::Orientation::Horizontal);
    left_sep.set_hexpand(true);
    left_sep.set_valign(gtk::Align::Center);

    let title_lbl = gtk::Label::builder()
        .label(&gettext("IN THIS CHAPTER").to_uppercase())
        .build();
    title_lbl.add_css_class("caption");
    title_lbl.add_css_class("dim-label");

    let right_sep = gtk::Separator::new(gtk::Orientation::Horizontal);
    right_sep.set_hexpand(true);
    right_sep.set_valign(gtk::Align::Center);

    header_row.append(&left_sep);
    header_row.append(&title_lbl);
    header_row.append(&right_sep);
    footer.append(&header_row);

    let mut grouped: BTreeMap<String, Vec<EntityRef>> = BTreeMap::new();
    for entity in entities {
        grouped
            .entry(entity.kind.to_lowercase())
            .or_default()
            .push(entity.clone());
    }

    for (kind, refs) in grouped {
        let kind_row = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(10)
            .halign(gtk::Align::Fill)
            .build();

        let kind_lbl = gtk::Label::builder()
            .label(&kind_label(&kind))
            .width_request(76)
            .halign(gtk::Align::Start)
            .valign(gtk::Align::Center)
            .build();
        kind_lbl.add_css_class("caption");
        kind_lbl.add_css_class("dim-label");

        let flow = gtk::FlowBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .homogeneous(false)
            .hexpand(true)
            .column_spacing(6)
            .row_spacing(6)
            .build();

        for entity in refs {
            let chip_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Horizontal)
                .spacing(6)
                .build();

            let badge = gtk::Label::builder()
                .label(&entity_initials(&entity.name))
                .build();
            badge.add_css_class("caption");

            let name_lbl = gtk::Label::builder().label(&entity.name).build();
            name_lbl.add_css_class("caption");

            chip_box.append(&badge);
            chip_box.append(&name_lbl);

            let btn = gtk::Button::builder().child(&chip_box).build();
            btn.add_css_class("pill");
            btn.add_css_class("flat");

            let cb = Rc::clone(&callback);
            let ent = entity.clone();
            btn.connect_clicked(move |_| {
                cb(ChapterViewEvent::EntitySelected(ent.clone()));
            });

            flow.insert(&btn, -1);
        }

        kind_row.append(&kind_lbl);
        kind_row.append(&flow);
        footer.append(&kind_row);
    }

    footer
}

fn kind_label(kind: &str) -> String {
    match kind {
        "person" => gettext("People"),
        "place" => gettext("Places"),
        "event" => gettext("Events"),
        "group" => gettext("Groups"),
        _ => gettext("Other"),
    }
}

fn entity_initials(name: &str) -> String {
    let mut parts = name.split_whitespace();
    let first = parts.next().and_then(|w| w.chars().next());
    let second = parts.next().and_then(|w| w.chars().next());

    match (first, second) {
        (Some(a), Some(b)) => format!("{}{}", a.to_uppercase(), b.to_uppercase()),
        (Some(a), None) => a.to_uppercase().to_string(),
        _ => String::new(),
    }
}
