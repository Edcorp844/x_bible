use adw::prelude::*;
use gettextrs::gettext;
use xbible_engine::features::reader::CommentaryChapter;
use xbible_engine::features::store::StoreItem;

use crate::core::ui::text_config::TextConfig;

pub(crate) fn build_commentary_panel<F>(
    commentaries: &[StoreItem],
    selected_id: Option<&str>,
    commentary_name: &str,
    chapter: Option<&CommentaryChapter>,
    is_loading: bool,
    config: &TextConfig,
    on_select_commentary: F,
) -> gtk::Box
where
    F: Fn(String, String) + 'static,
{
    let panel = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(0)
        .hexpand(true)
        .vexpand(true)
        .build();

    // Header bar with commentary selector
    let header = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(10)
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(16)
        .margin_end(16)
        .build();

    if commentaries.is_empty() {
        let lbl = gtk::Label::builder()
            .label(&gettext("Select Commentary"))
            .halign(gtk::Align::Start)
            .build();
        lbl.add_css_class("heading");
        lbl.add_css_class("dim-label");
        header.append(&lbl);
    } else {
        let labels: Vec<&str> = commentaries.iter().map(|c| c.name.as_str()).collect();
        let dropdown = gtk::DropDown::from_strings(&labels);
        dropdown.set_hexpand(false);

        let initial_idx = selected_id
            .and_then(|id| commentaries.iter().position(|c| c.id == id))
            .unwrap_or(0) as u32;
        dropdown.set_selected(initial_idx);

        let items_owned: Vec<(String, String)> = commentaries
            .iter()
            .map(|c| (c.id.clone(), c.name.clone()))
            .collect();
        dropdown.connect_selected_notify(move |dd| {
            let idx = dd.selected() as usize;
            if let Some((id, name)) = items_owned.get(idx) {
                on_select_commentary(id.clone(), name.clone());
            }
        });

        header.append(&dropdown);
    }

    panel.append(&header);
    panel.append(&gtk::Separator::new(gtk::Orientation::Horizontal));

    let scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .vexpand(true)
        .hexpand(true)
        .build();

    if commentaries.is_empty() {
        let status = adw::StatusPage::builder()
            .icon_name("dictionary-symbolic")
            .title(&gettext("No Commentaries Installed"))
            .description(&gettext(
                "Install a commentary from the Bible Store to read it here.",
            ))
            .build();
        scroll.set_child(Some(&status));
    } else if is_loading {
        let spinner_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .halign(gtk::Align::Center)
            .valign(gtk::Align::Center)
            .vexpand(true)
            .build();
        let spinner = gtk::Spinner::new();
        spinner.set_spinning(true);
        spinner.set_width_request(32);
        spinner.set_height_request(32);
        spinner_box.append(&spinner);
        scroll.set_child(Some(&spinner_box));
    } else if let Some(data) = chapter {
        let body = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(12)
            .margin_top(16)
            .margin_bottom(24)
            .margin_start(20)
            .margin_end(20)
            .build();

        let title_text = if commentary_name.is_empty() {
            format!("{} {}", data.book_name, data.chapter_number)
        } else {
            format!(
                "{commentary_name} · {} {}",
                data.book_name, data.chapter_number
            )
        };
        let title_lbl = gtk::Label::builder()
            .label(&title_text)
            .halign(gtk::Align::Start)
            .wrap(true)
            .build();
        title_lbl.add_css_class("title-3");
        body.append(&title_lbl);

        let font_px =
            ((17.0 * (config.font_size_percent.clamp(50, 300) as f64 / 100.0)) * 1.333) as i32;
        for block in &data.blocks {
            let b_lbl = gtk::Label::builder()
                .label(&block.text)
                .halign(gtk::Align::Fill)
                .wrap(true)
                .justify(if config.justify {
                    gtk::Justification::Fill
                } else {
                    gtk::Justification::Left
                })
                .build();
            let attrs = gtk::pango::AttrList::new();
            attrs.insert(gtk::pango::AttrSize::new(font_px * gtk::pango::SCALE));
            attrs.insert(gtk::pango::AttrString::new_family(
                config.resolved_font_family(),
            ));
            b_lbl.set_attributes(Some(&attrs));
            body.append(&b_lbl);
        }

        scroll.set_child(Some(&body));
    } else {
        let status = adw::StatusPage::builder()
            .icon_name("document-properties-symbolic")
            .title(&gettext("No Notes"))
            .description(&gettext(
                "No notes for this chapter in the selected commentary.",
            ))
            .build();
        scroll.set_child(Some(&status));
    }

    panel.append(&scroll);
    panel
}
