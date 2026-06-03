use adw::prelude::*;
use relm4::{ComponentSender, prelude::*};
use xbible_engine::engines::module_engine::sword_module::{
    module::SwordModule, module_book::ModuleBook,
};

use crate::features::bible::components::page::biblepage_model::{
    BiblePage, BiblePageWidgets, StudyInput,
};

impl BiblePage {
    pub(crate) fn populate_version_grid(
        widgets: &BiblePageWidgets,
        modules: &[SwordModule],
        sender: ComponentSender<Self>,
    ) {
        // 1. Instant Clear
        while let Some(child) = widgets.bible_grid.first_child() {
            widgets.bible_grid.remove(&child);
        }

        // 2. Grouping (Still fast on main thread)
        let mut grouped: std::collections::BTreeMap<String, Vec<SwordModule>> =
            std::collections::BTreeMap::new();
        for module in modules {
            grouped
                .entry(module.language.clone())
                .or_default()
                .push(module.clone());
        }

        // Convert to a flat list of tasks for the idle loop
        let mut tasks: std::collections::VecDeque<(String, Vec<SwordModule>)> =
            grouped.into_iter().collect();

        let grid = widgets.bible_grid.clone();
        let pop = widgets.version_popover.clone();
        let s = sender.clone();

        // 3. Idle Slicer for Versions
        glib::idle_add_local(move || {
            if let Some((lang, lang_modules)) = tasks.pop_front() {
                // Header
                let header_label = gtk::Label::builder()
                    .halign(gtk::Align::Start)
                    .margin_top(20)
                    .margin_bottom(12)
                    .margin_start(16)
                    .build();

                header_label.set_markup(&format!(
                    "<span size='small' weight='heavy' alpha='60%' letter_spacing='1200'>{}</span>",
                    lang.to_uppercase()
                ));
                grid.append(&header_label);

                // WrapBox
                let wrap_box = adw::WrapBox::builder()
                    .orientation(gtk::Orientation::Horizontal)
                    .line_spacing(5)
                    .child_spacing(5)
                    .margin_start(10)
                    .margin_end(10)
                    .margin_bottom(20)
                    .build();

                for module in lang_modules {
                    let tile = Self::create_bible_tile(&module.name, &module.language);
                    let s_inner = s.clone();
                    let m_inner = module.clone();
                    let p_inner = pop.clone();

                    tile.connect_clicked(move |_| {
                        s_inner.input(StudyInput::SetModule(m_inner.clone()));
                        p_inner.popdown();
                    });
                    wrap_box.append(&tile);
                }
                grid.append(&wrap_box);

                // Separator
                let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
                sep.set_opacity(0.1);
                sep.set_margin_start(16);
                sep.set_margin_end(16);
                grid.append(&sep);

                glib::ControlFlow::Continue
            } else {
                glib::ControlFlow::Break
            }
        });
    }

    pub(crate) fn populate_book_grid(
        &self,
        widgets: &BiblePageWidgets,
        books: &Vec<ModuleBook>,
        sender: ComponentSender<Self>,
    ) {
        // Instant Clear
        while let Some(child) = widgets.ot_grid.first_child() {
            widgets.ot_grid.remove(&child);
        }
        while let Some(child) = widgets.nt_grid.first_child() {
            widgets.nt_grid.remove(&child);
        }

        let mut books_queue: std::collections::VecDeque<ModuleBook> = books.to_vec().into();

        let ot_grid = widgets.ot_grid.clone();
        let nt_grid = widgets.nt_grid.clone();
        let ot_cont = widgets.ot_container.clone();
        let nt_cont = widgets.nt_container.clone();
        let pop = widgets.book_popover.clone();
        let s = sender.clone();

        // Idle Slicer for Books (8 books per frame)
        glib::idle_add_local(move || {
            for _ in 0..8 {
                if let Some(book) = books_queue.pop_front() {
                    let btn = Self::create_book_tile(&book.name);
                    let s_inner = s.clone();
                    let p_inner = pop.clone();
                    let book_for_click = book.clone();

                    btn.connect_clicked(move |_| {
                        s_inner.input(StudyInput::SetBook(book_for_click.clone()));
                        p_inner.popdown();
                    });

                    // Determine if book is OT or NT
                    // Common OT books (1-39 in traditional order)
                    let ot_books = vec![
                        "Genesis",
                        "Exodus",
                        "Leviticus",
                        "Numbers",
                        "Deuteronomy",
                        "Joshua",
                        "Judges",
                        "Ruth",
                        "1 Samuel",
                        "2 Samuel",
                        "1 Kings",
                        "2 Kings",
                        "1 Chronicles",
                        "2 Chronicles",
                        "Ezra",
                        "Nehemiah",
                        "Esther",
                        "Job",
                        "Psalm",
                        "Psalms",
                        "Proverbs",
                        "Ecclesiastes",
                        "Isaiah",
                        "Jeremiah",
                        "Lamentations",
                        "Ezekiel",
                        "Daniel",
                        "Hosea",
                        "Joel",
                        "Amos",
                        "Obadiah",
                        "Jonah",
                        "Micah",
                        "Nahum",
                        "Habakkuk",
                        "Zephaniah",
                        "Haggai",
                        "Zechariah",
                        "Malachi",
                    ];

                    let is_ot = ot_books.iter().any(|&b| book.name.starts_with(b));

                    if is_ot {
                        ot_grid.append(&btn);
                        ot_cont.set_visible(true);
                    } else {
                        nt_grid.append(&btn);
                        nt_cont.set_visible(true);
                    }
                } else {
                    return glib::ControlFlow::Break;
                }
            }
            glib::ControlFlow::Continue
        });
    }

    pub(crate) fn populate_chapter_grid(
        &self,
        widgets: &BiblePageWidgets,
        sender: ComponentSender<Self>,
        count: i32, // Pass the count in directly
    ) {
        while let Some(child) = widgets.chapter_grid.first_child() {
            widgets.chapter_grid.remove(&child);
        }

        let mut current_idx = 1;
        let grid = widgets.chapter_grid.clone();
        let pop = widgets.chapter_popover.clone();
        let s = sender.clone();

        glib::idle_add_local(move || {
            for _ in 0..12 {
                if current_idx <= count {
                    let btn = gtk::Button::builder()
                        .label(&current_idx.to_string())
                        .css_classes(vec!["card", "chapter-tile"])
                        .build();

                    let s_inner = s.clone();
                    let p_inner = pop.clone();
                    let val = current_idx;

                    btn.connect_clicked(move |_| {
                        s_inner.input(StudyInput::SetChapter(val as i32));
                        p_inner.popdown();
                    });

                    grid.append(&btn);
                    current_idx += 1;
                } else {
                    return glib::ControlFlow::Break;
                }
            }
            glib::ControlFlow::Continue
        });
    }

    pub(crate) fn create_bible_tile(name: &str, _lang: &str) -> gtk::Button {
        let button = gtk::Button::builder()
            .width_request(120)
            .height_request(48)
            .css_classes(vec!["card", "flat"])
            .build();

        let label = gtk::Label::builder().build();
        label.set_markup(&format!(
            "<span weight='bold' size='medium' font_features='tnum'>{}</span>",
            name
        ));

        button.set_child(Some(&label));
        button.set_margin_all(2);
        button
    }

    pub(crate) fn create_book_tile(name: &str) -> gtk::Button {
        gtk::Button::builder()
            .label(name)
            .css_classes(vec!["card", "book-tile"])
            .width_request(85)
            .height_request(40)
            .build()
    }
}
