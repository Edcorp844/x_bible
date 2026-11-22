use adw::prelude::*;
use gettextrs::gettext;
use xbible_engine::features::reader::{
    BookSection,
    search::{SearchHit, SearchResults},
};

use crate::workers::reader_worker::{CanonSection, SearchType};

#[derive(Debug, Clone)]
pub(crate) enum SearchPanelEvent {
    QueryChanged(String),
    ClearQuery,
    SearchTypeChanged(SearchType),
    ResetAllSections,
    ToggleSection(CanonSection),
    BookFilterChanged(Option<String>),
    HitSelected(SearchHit),
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_search_panel<F>(
    query: &str,
    search_type: SearchType,
    selected_sections: &std::collections::HashSet<CanonSection>,
    selected_book_id: Option<&str>,
    sections: &[BookSection],
    is_searching: bool,
    progress_fraction: f64,
    progress_message: &str,
    results: Option<&SearchResults>,
    error: Option<&str>,
    on_event: F,
) -> gtk::Box
where
    F: Fn(SearchPanelEvent) + 'static,
{
    let callback = std::rc::Rc::new(on_event);
    let container = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(0)
        .hexpand(true)
        .vexpand(true)
        .build();

    // Header: SearchEntry + Filter Row
    let header_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(10)
        .margin_top(12)
        .margin_bottom(10)
        .margin_start(16)
        .margin_end(16)
        .build();

    let entry = gtk::SearchEntry::builder()
        .placeholder_text(&gettext(
            "Search verses, references, or Strong's (e.g. JHN 3:16, H430)",
        ))
        .text(query)
        .hexpand(true)
        .build();
    {
        let cb = std::rc::Rc::clone(&callback);
        entry.connect_search_changed(move |e| {
            let text = e.text().to_string();
            if text.trim().is_empty() {
                cb(SearchPanelEvent::ClearQuery);
            } else {
                cb(SearchPanelEvent::QueryChanged(text));
            }
        });
    }
    header_box.append(&entry);

    // Filter row: SearchType, CanonSection menu, Book filter menu
    let filter_row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();

    let modes = [
        (SearchType::MultiWord, gettext("Multi Word")),
        (SearchType::Phrase, gettext("Phrase")),
        (SearchType::RegularExpression, gettext("Regex")),
    ];
    let mode_labels: Vec<&str> = modes.iter().map(|(_, l)| l.as_str()).collect();
    let mode_dropdown = gtk::DropDown::from_strings(&mode_labels);
    let mode_idx = modes
        .iter()
        .position(|(m, _)| *m == search_type)
        .unwrap_or(0) as u32;
    mode_dropdown.set_selected(mode_idx);
    {
        let cb = std::rc::Rc::clone(&callback);
        let mode_list: Vec<SearchType> = modes.iter().map(|(m, _)| *m).collect();
        mode_dropdown.connect_selected_notify(move |dd| {
            if let Some(m) = mode_list.get(dd.selected() as usize) {
                cb(SearchPanelEvent::SearchTypeChanged(*m));
            }
        });
    }
    filter_row.append(&mode_dropdown);

    // Section filter MenuButton
    let all_selected = selected_sections.len() == CanonSection::ALL.len();
    let section_btn_label = if all_selected {
        gettext("All Sections")
    } else if selected_sections.len() == 1 {
        let wire = selected_sections
            .iter()
            .next()
            .map(|s| s.wire())
            .unwrap_or("unknown");
        wire.to_uppercase()
    } else {
        format!("{} {}", selected_sections.len(), gettext("Sections"))
    };

    let section_menu_btn = gtk::MenuButton::builder().label(&section_btn_label).build();
    section_menu_btn.add_css_class("pill");

    let sec_popover = gtk::Popover::new();
    let sec_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(4)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .build();

    let all_btn = gtk::Button::builder()
        .label(&gettext("All Sections"))
        .halign(gtk::Align::Start)
        .build();
    all_btn.add_css_class("flat");
    {
        let pop_weak = sec_popover.downgrade();
        let cb = std::rc::Rc::clone(&callback);
        all_btn.connect_clicked(move |_| {
            cb(SearchPanelEvent::ResetAllSections);
            if let Some(pop) = pop_weak.upgrade() {
                pop.popdown();
            }
        });
    }
    sec_box.append(&all_btn);
    sec_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    for sec in CanonSection::ALL {
        let is_on = selected_sections.contains(&sec);
        let check = gtk::CheckButton::builder()
            .label(sec.wire())
            .active(is_on)
            .sensitive(!(is_on && selected_sections.len() == 1))
            .build();
        let cb = std::rc::Rc::clone(&callback);
        check.connect_toggled(move |_| {
            cb(SearchPanelEvent::ToggleSection(sec));
        });
        sec_box.append(&check);
    }
    sec_popover.set_child(Some(&sec_box));
    section_menu_btn.set_popover(Some(&sec_popover));
    filter_row.append(&section_menu_btn);

    // Book filter MenuButton
    let matching_books: Vec<(String, String)> = sections
        .iter()
        .filter(|g| selected_sections.contains(&CanonSection::from_wire(&g.section_key)))
        .flat_map(|g| g.books.iter().map(|b| (b.id.clone(), b.name.clone())))
        .collect();

    let current_book_name = selected_book_id
        .and_then(|bid| matching_books.iter().find(|(id, _)| id == bid))
        .map(|(_, name)| name.clone())
        .unwrap_or_else(|| gettext("All Books"));

    let book_menu_btn = gtk::MenuButton::builder()
        .label(&current_book_name)
        .sensitive(!matching_books.is_empty())
        .build();
    book_menu_btn.add_css_class("pill");

    let book_popover = gtk::Popover::new();
    let book_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(2)
        .margin_top(8)
        .margin_bottom(8)
        .margin_start(8)
        .margin_end(8)
        .build();

    let all_books_btn = gtk::Button::builder()
        .label(&gettext("All Books"))
        .halign(gtk::Align::Start)
        .build();
    all_books_btn.add_css_class("flat");
    {
        let pop_weak = book_popover.downgrade();
        let cb = std::rc::Rc::clone(&callback);
        all_books_btn.connect_clicked(move |_| {
            cb(SearchPanelEvent::BookFilterChanged(None));
            if let Some(pop) = pop_weak.upgrade() {
                pop.popdown();
            }
        });
    }
    book_box.append(&all_books_btn);

    for (b_id, b_name) in matching_books {
        let btn = gtk::Button::builder()
            .label(&b_name)
            .halign(gtk::Align::Start)
            .build();
        btn.add_css_class("flat");
        let pop_weak = book_popover.downgrade();
        let cb = std::rc::Rc::clone(&callback);
        let id_owned = b_id.clone();
        btn.connect_clicked(move |_| {
            cb(SearchPanelEvent::BookFilterChanged(Some(id_owned.clone())));
            if let Some(pop) = pop_weak.upgrade() {
                pop.popdown();
            }
        });
        book_box.append(&btn);
    }

    book_popover.set_child(Some(&book_box));
    book_menu_btn.set_popover(Some(&book_popover));
    filter_row.append(&book_menu_btn);

    header_box.append(&filter_row);
    container.append(&header_box);
    container.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    // Body states
    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .hexpand(true)
        .build();

    if let Some(err_msg) = error {
        let status = adw::StatusPage::builder()
            .icon_name("dialog-warning-symbolic")
            .title(&gettext("Search failed"))
            .description(err_msg)
            .build();
        scroll.set_child(Some(&status));
    } else if is_searching {
        let prog_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(12)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .vexpand(true)
            .margin_top(24)
            .margin_bottom(24)
            .build();

        let pct = (progress_fraction.clamp(0.0, 1.0) * 100.0).round() as i32;
        let pct_lbl = gtk::Label::builder().label(&format!("{pct}%")).build();
        pct_lbl.add_css_class("heading");

        let bar = gtk::ProgressBar::builder()
            .fraction(progress_fraction.clamp(0.0, 1.0))
            .width_request(220)
            .build();

        let search_label = gettext("Searching…");
        let msg_lbl = gtk::Label::builder()
            .label(if progress_message.trim().is_empty() {
                &search_label
            } else {
                progress_message
            })
            .wrap(true)
            .build();
        msg_lbl.add_css_class("dim-label");

        prog_box.append(&pct_lbl);
        prog_box.append(&bar);
        prog_box.append(&msg_lbl);
        scroll.set_child(Some(&prog_box));
    } else if query.trim().is_empty() {
        let status = adw::StatusPage::builder()
            .icon_name("edit-find-symbolic")
            .title(&gettext("Search"))
            .description(&gettext(
                "Search text (\"shepherd\"), a reference (\"JHN 3:16\"), or a Strong's number (\"H430\").",
            ))
            .build();
        scroll.set_child(Some(&status));
    } else if let Some(res) = results {
        let filtered_hits: Vec<&SearchHit> = res
            .hits
            .iter()
            .filter(|h| selected_book_id.is_none_or(|bid| h.book_id == bid))
            .collect();

        if filtered_hits.is_empty() {
            let status = adw::StatusPage::builder()
                .icon_name("edit-find-symbolic")
                .title(&gettext("No results"))
                .description(
                    &gettext("No verses matched \"{}\" in the selected translation.")
                        .replace("{}", query.trim()),
                )
                .build();
            scroll.set_child(Some(&status));
        } else {
            let list = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .spacing(8)
                .margin_top(12)
                .margin_bottom(24)
                .margin_start(16)
                .margin_end(16)
                .build();

            for hit in filtered_hits {
                let row_box = gtk::Box::builder()
                    .orientation(gtk::Orientation::Vertical)
                    .spacing(4)
                    .halign(gtk::Align::Fill)
                    .build();

                let ref_lbl = gtk::Label::builder()
                    .label(&format!("{} {}:{}", hit.book_name, hit.chapter, hit.verse))
                    .halign(gtk::Align::Start)
                    .build();
                ref_lbl.add_css_class("caption");
                ref_lbl.add_css_class("accent");

                let snippet_lbl = gtk::Label::builder()
                    .label(&hit.text)
                    .halign(gtk::Align::Fill)
                    .wrap(true)
                    .build();
                snippet_lbl.set_attributes(Some(&build_hit_highlight_attrs(hit)));

                row_box.append(&ref_lbl);
                row_box.append(&snippet_lbl);

                let btn = gtk::Button::builder().child(&row_box).build();
                btn.add_css_class("flat");

                let cb = std::rc::Rc::clone(&callback);
                let hit_owned = hit.clone();
                btn.connect_clicked(move |_| {
                    cb(SearchPanelEvent::HitSelected(hit_owned.clone()));
                });

                list.append(&btn);
            }

            scroll.set_child(Some(&list));
        }
    }

    container.append(&scroll);
    container
}

fn build_hit_highlight_attrs(hit: &SearchHit) -> gtk::pango::AttrList {
    let attrs = gtk::pango::AttrList::new();
    let bytes = hit.text.as_bytes();

    for span in &hit.highlights {
        let start = span.start.max(0) as usize;
        let end = span.end.max(0) as usize;
        if start >= end || end > bytes.len() {
            continue;
        }
        if !hit.text.is_char_boundary(start) || !hit.text.is_char_boundary(end) {
            continue;
        }

        let char_start = hit.text[..start].chars().count() as u32;
        let char_end = hit.text[..end].chars().count() as u32;

        let mut weight_attr = gtk::pango::AttrInt::new_weight(gtk::pango::Weight::Bold);
        weight_attr.set_start_index(char_start);
        weight_attr.set_end_index(char_end);
        attrs.insert(weight_attr);

        if let Ok(rgba) = gtk::gdk::RGBA::parse("#3584E438") {
            let mut bg_attr = gtk::pango::AttrColor::new_background(
                (rgba.red() * 65535.0) as u16,
                (rgba.green() * 65535.0) as u16,
                (rgba.blue() * 65535.0) as u16,
            );
            bg_attr.set_start_index(char_start);
            bg_attr.set_end_index(char_end);
            attrs.insert(bg_attr);
        }
    }

    attrs
}
