use std::cell::Cell;
use std::rc::Rc;
use std::time::Duration;

use adw::prelude::*;
use gtk::prelude::*;
use gtk::{gdk, gio, glib, pango};
use relm4::{Component, ComponentParts};
use xbible_engine::{
    data::timeline_data::data::TimelineData,
    data::timeline_data::structs::{Event, Period},
    engines::xbible_engine::engine::XBibleEngine,
};

pub struct TimelinePage {
    pub(crate) engine: Rc<XBibleEngine>,
}

#[derive(Clone, Debug)]
pub enum TimelinePageInput {}

#[derive(Clone, Debug)]
pub enum TimelinePageOutput {
    ToggleSidebar,
}

// ---- Layout: base values that get SCALED to the viewport, not used directly ----
const TIMELINE_START_YEAR: i64 = -4100;
const TIMELINE_END_YEAR: i64 = 2100;
const INITIAL_YEAR: i64 = 2026; // where the scrubber starts on first load, nothing more

const BASE_PIXELS_PER_YEAR: f64 = 1.2;
const BASE_ROW_HEIGHT: f64 = 60.0;
const BASE_MIN_EVENT_WIDTH: f64 = 90.0;
const REFERENCE_WIDTH: f64 = 1200.0; // window width the BASE_* constants were tuned for
const MIN_SCALE: f64 = 0.6;
const MAX_SCALE: f64 = 2.2;

const HEADER_HEIGHT: f64 = 32.0;
const MAX_ROWS: f64 = 26.0;
const RESIZE_DEBOUNCE_MS: u64 = 50;

#[relm4::component(pub)]
impl Component for TimelinePage {
    type Init = std::sync::Arc<XBibleEngine>;
    type Input = TimelinePageInput;
    type Output = TimelinePageOutput;
    type CommandOutput = ();

    fn init(
        engine: Self::Init,
        root: Self::Root,
        sender: relm4::prelude::ComponentSender<Self>,
    ) -> relm4::prelude::ComponentParts<Self> {
        let model = TimelinePage {
            engine: Rc::new((*engine).clone()),
        };
        let widgets = view_output!();

        let periods = TimelineData::new().get_data();
        let timeline_widget = build_timeline(periods);
        widgets.timeline_container.append(&timeline_widget);

        ComponentParts { model, widgets }
    }

    view! {
        #[root]
        adw::NavigationPage {
            set_title: "Timeline",

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle { set_title: "Timeline" },
                    set_show_title: false,
                    add_css_class: "flat",

                    pack_start = &gtk::ToggleButton {
                        set_icon_name: "sidebar-show-symbolic",
                        connect_clicked[sender] => move |_| {
                            let _ = sender.output(TimelinePageOutput::ToggleSidebar);
                        }
                    }
                },

                #[wrap(Some)]
                #[name = "timeline_container"]
                set_content = &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_vexpand: true,
                    set_hexpand: true,
                }
            }
        }
    }
}

// ---- Helpers ----

fn year_to_x(year: i64, pixels_per_year: f64) -> f64 {
    (year - TIMELINE_START_YEAR) as f64 * pixels_per_year
}

fn year_label(year: i64) -> String {
    if year < 0 {
        format!("{} BC", -year)
    } else if year == 0 {
        "0".to_string()
    } else {
        format!("{} AD", year)
    }
}

fn year_range_label(start: i64, end: i64) -> String {
    if start == end {
        year_label(start)
    } else {
        format!("{} – {}", year_label(start), year_label(end))
    }
}

/// Given the current scroll offset and scale, returns the year sitting
/// directly under the center scrubber line right now. This is the same
/// calculation your SwiftUI `updateYear(_:)` did from `scrollGeo.frame(...).minX`.
fn year_under_scrubber(hadjustment_value: f64, pixels_per_year: f64) -> i64 {
    if pixels_per_year <= 0.0 {
        return INITIAL_YEAR;
    }
    let raw = TIMELINE_START_YEAR as f64 + hadjustment_value / pixels_per_year;
    raw.round() as i64
}

/// Derives a scale factor from the live viewport width, clamped so things
/// never get absurdly tiny or huge. This replaces every hardcoded constant.
fn scale_for_width(width: f64) -> f64 {
    (width / REFERENCE_WIDTH).clamp(MIN_SCALE, MAX_SCALE)
}

/// Removes every child from a Fixed so it can be repopulated at a new scale.
fn clear_fixed(fixed: &gtk::Fixed) {
    while let Some(child) = fixed.first_child() {
        fixed.remove(&child);
    }
}

/// (Re)populates the year ruler at the given scale.
fn layout_header(header_fixed: &gtk::Fixed, pixels_per_year: f64, center_x: f64) {
    clear_fixed(header_fixed);
    let total_width = year_to_x(TIMELINE_END_YEAR, pixels_per_year) + center_x * 2.0;
    header_fixed.set_size_request(total_width as i32, HEADER_HEIGHT as i32);

    let mut year = TIMELINE_START_YEAR;
    while year < TIMELINE_END_YEAR {
        let label = gtk::Label::new(Some(&year_label(year)));
        label.add_css_class("dim-label");
        label.add_css_class("caption");
        header_fixed.put(&label, center_x + year_to_x(year, pixels_per_year), 6.0);
        year += 100;
    }
}

/// (Re)populates the grid lines + event cards at the given scale.
fn layout_content(
    content_fixed: &gtk::Fixed,
    periods: &[Period],
    pixels_per_year: f64,
    row_height: f64,
    center_x: f64,
    on_event_click: &Rc<dyn Fn(&gtk::Window, &Event)>,
) {
    clear_fixed(content_fixed);

    let content_height = row_height * MAX_ROWS + HEADER_HEIGHT;
    let total_width = year_to_x(TIMELINE_END_YEAR, pixels_per_year) + center_x * 2.0;
    content_fixed.set_size_request(total_width as i32, content_height as i32);

    // Grid lines
    let mut gy = TIMELINE_START_YEAR;
    while gy < TIMELINE_END_YEAR {
        let line = gtk::Separator::new(gtk::Orientation::Vertical);
        line.add_css_class("timeline-grid-line");
        line.set_size_request(1, content_height as i32);
        content_fixed.put(&line, center_x + year_to_x(gy, pixels_per_year), 0.0);
        gy += 100;
    }

    let min_event_width = BASE_MIN_EVENT_WIDTH * (pixels_per_year / BASE_PIXELS_PER_YEAR);

    for period in periods {
        let class_name = format!("event-color-{}", period.id);
        for event in &period.events {
            let start_x = year_to_x(event.start, pixels_per_year);
            let end_x = year_to_x(event.end, pixels_per_year);
            let width = (end_x - start_x).max(0.0).max(min_event_width);

            let card = build_event_card(event, &class_name);
            card.set_size_request(width as i32, (row_height - 10.0) as i32);

            let y = event.row as f64 * row_height + HEADER_HEIGHT + 4.0;
            content_fixed.put(&card, center_x + start_x, y);

            let event_clone = event.clone();
            let on_click = on_event_click.clone();
            card.connect_clicked(move |btn| {
                if let Some(window) = btn.root().and_downcast::<gtk::Window>() {
                    on_click(&window, &event_clone);
                }
            });
        }
    }
}

/// Builds the whole scrollable timeline. Structure:
///
///   Overlay
///     base:     probe (invisible, just reports live viewport size)
///     overlay:  inner_box = [header_scroller, separator, content_scroller]
///     overlay:  scrubber  = [label, vertical line] — valign: Fill, so the
///               line runs the full height, through the header AND the
///               content, giving precision between the 100-year labels.
///
/// The scrubber's label text is recomputed on every `hadjustment` scroll tick
/// via `year_under_scrubber`, not a fixed constant — it always reflects
/// whatever year currently sits under the line, exactly like your SwiftUI
/// `activeYear` state.
fn build_timeline(periods: Vec<Period>) -> gtk::Widget {
    let periods = Rc::new(periods);

    // --- one-time CSS, independent of size ---
    let mut css = String::from(
        "
        .event-card { border-width: 1px; border-style: solid; border-radius: 8px; padding: 2px 8px 2px 0; }
        .event-accent { min-width: 4px; border-radius: 8px 0 0 8px; }
        .event-button { padding: 0; }
        .timeline-grid-line { opacity: 0.35; }
        .scrubber-line { background-color: #e01b24;}
        .scrubber-bubble { background-color: #e01b24; color: white; border-radius: 999px; padding: 3px 10px; font-weight: bold; }
        ",
    );
    for period in periods.iter() {
        css.push_str(&format!(
            ".event-color-{id} {{ border-color: {col}; }} .event-color-{id} .event-accent {{ background-color: {col}; }} .event-color-{id} .event-year {{ color: {col}; }}\n",
            id = period.id, col = period.color,
        ));
    }
    let css_provider = gtk::CssProvider::new();
    css_provider.load_from_data(&css);
    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("no display connection"),
        &css_provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // upper must be >= page_size at construction, or gtk_adjustment_new
    // returns NULL and gtk4-rs panics on the null pointer. ScrolledWindow
    // replaces all of these once the Fixed child is realized and reports
    // its real size, so this just needs to be internally consistent.
    let initial_upper = year_to_x(TIMELINE_END_YEAR, BASE_PIXELS_PER_YEAR).max(1.0);
    let hadjustment = gtk::Adjustment::new(0.0, 0.0, initial_upper, 40.0, 400.0, 1.0);

    // --- header (year ruler), horizontal scroll only, synced ---
    let header_fixed = gtk::Fixed::new();
    let header_scroller = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::External)
        .vscrollbar_policy(gtk::PolicyType::Never)
        .hadjustment(&hadjustment)
        .build();
    header_scroller.set_child(Some(&header_fixed));
    header_scroller.set_size_request(-1, HEADER_HEIGHT as i32 + 6);

    // --- content canvas (grid + event cards), scrolls both ways ---
    let content_fixed = gtk::Fixed::new();
    let content_scroller = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Automatic)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .hadjustment(&hadjustment)
        .hexpand(true)
        .vexpand(true)
        .build();
    content_scroller.set_child(Some(&content_fixed));

    // --- stack header + content so the scrubber can overlay both at once ---
    let inner_box = gtk::Box::new(gtk::Orientation::Vertical, 0);
    inner_box.set_hexpand(true);
    inner_box.set_vexpand(true);
    inner_box.append(&header_scroller);
    inner_box.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
    inner_box.append(&content_scroller);

    // --- size probe: purely for reading the live viewport width ---
    let probe = gtk::DrawingArea::new();
    probe.set_hexpand(true);
    probe.set_vexpand(true);

    // --- overlay: probe decides overlay size, inner_box fills it, scrubber pinned to center, full height ---
    let overlay = gtk::Overlay::new();
    overlay.set_hexpand(true);
    overlay.set_vexpand(true);
    overlay.set_child(Some(&probe));
    overlay.add_overlay(&inner_box);

    let scrubber_label = gtk::Label::new(Some(&year_label(INITIAL_YEAR)));
    scrubber_label.add_css_class("scrubber-bubble");
    scrubber_label.add_css_class("caption");
    scrubber_label.set_halign(gtk::Align::Center);

    let scrubber_line = gtk::Box::new(gtk::Orientation::Vertical, 0);
    scrubber_line.add_css_class("scrubber-line");
    scrubber_line.set_width_request(1);
    scrubber_line.set_halign(gtk::Align::Center);
    scrubber_line.set_vexpand(true); // stretches the line the rest of the way down

    let scrubber = gtk::Box::new(gtk::Orientation::Vertical, 4);
    scrubber.append(&scrubber_label);
    scrubber.append(&scrubber_line);
    scrubber.set_halign(gtk::Align::Center);
    scrubber.set_valign(gtk::Align::Fill); // top-to-bottom, through header AND content
    scrubber.set_can_target(false); // clicks pass through, like allowsHitTesting(false)
    overlay.add_overlay(&scrubber);

    // --- event click handler ---
    let on_event_click: Rc<dyn Fn(&gtk::Window, &Event)> = Rc::new(|window, event| {
        show_event_detail(
            window,
            &event.title,
            &event.slug,
            event.start,
            event.end,
            event.image.clone(),
        );
    });

    // --- state carried across resizes / scroll ---
    let current_ppy = Rc::new(Cell::new(BASE_PIXELS_PER_YEAR));
    let did_initial_center = Rc::new(Cell::new(false));
    let pending_resize: Rc<Cell<Option<glib::SourceId>>> = Rc::new(Cell::new(None));

    // Keep the scrubber label in sync with whatever year is actually under
    // the line right now — fires on every scroll/drag/programmatic move.
    {
        let scrubber_label = scrubber_label.clone();
        let current_ppy = current_ppy.clone();
        hadjustment.connect_value_changed(move |adj| {
            let year = year_under_scrubber(adj.value(), current_ppy.get());
            scrubber_label.set_text(&year_label(year));
        });
    }

    {
        let periods = periods.clone();
        let header_fixed = header_fixed.clone();
        let content_fixed = content_fixed.clone();
        let hadjustment = hadjustment.clone();
        let current_ppy = current_ppy.clone();
        let did_initial_center = did_initial_center.clone();
        let on_event_click = on_event_click.clone();
        let pending_resize = pending_resize.clone();

        probe.connect_resize(move |_, width, _height| {
            let width = width as f64;
            if width <= 0.0 {
                return;
            }

            // Cancel any layout pass still waiting to run — only the last
            // resize in a burst actually gets applied.
            if let Some(id) = pending_resize.take() {
                id.remove();
            }

            let periods = periods.clone();
            let header_fixed = header_fixed.clone();
            let content_fixed = content_fixed.clone();
            let hadjustment = hadjustment.clone();
            let current_ppy = current_ppy.clone();
            let did_initial_center = did_initial_center.clone();
            let on_event_click = on_event_click.clone();
            let pending_resize_inner = pending_resize.clone();

            let source_id = glib::source::timeout_add_local(
                Duration::from_millis(RESIZE_DEBOUNCE_MS),
                move || {
                    // Preserve whichever year is currently under the scrubber
                    // so a resize doesn't yank the view back to INITIAL_YEAR
                    // once the user has scrolled.
                    let old_ppy = current_ppy.get();
                    let centered_year = if did_initial_center.get() {
                        year_under_scrubber(hadjustment.value(), old_ppy)
                    } else {
                        INITIAL_YEAR
                    };

                    let scale = scale_for_width(width);
                    let pixels_per_year = BASE_PIXELS_PER_YEAR * scale;
                    let row_height = BASE_ROW_HEIGHT * scale;
                    let center_x = width / 2.0;

                    layout_header(&header_fixed, pixels_per_year, center_x);
                    layout_content(
                        &content_fixed,
                        &periods,
                        pixels_per_year,
                        row_height,
                        center_x,
                        &on_event_click,
                    );

                    current_ppy.set(pixels_per_year);
                    did_initial_center.set(true);
                    // Triggers the value-changed handler above, which updates
                    // the scrubber label to match the new position.
                    hadjustment.set_value(year_to_x(centered_year, pixels_per_year));

                    pending_resize_inner.set(None);
                    glib::ControlFlow::Break
                },
            );

            pending_resize.set(Some(source_id));
        });
    }

    overlay.upcast()
}

/// One event "chip" — analogous to TimelineItemView.
fn build_event_card(event: &Event, color_class: &str) -> gtk::Button {
    let outer = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    outer.add_css_class("event-card");
    outer.add_css_class(color_class);

    let accent = gtk::Box::new(gtk::Orientation::Vertical, 0);
    accent.add_css_class("event-accent");
    outer.append(&accent);

    let text_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text_box.set_hexpand(true);
    text_box.set_margin_start(6);
    text_box.set_margin_top(4);
    text_box.set_margin_bottom(4);
    text_box.set_valign(gtk::Align::Center);

    let year_lbl = gtk::Label::new(Some(&year_range_label(event.start, event.end)));
    year_lbl.add_css_class("event-year");
    year_lbl.add_css_class("caption");
    year_lbl.set_xalign(0.0);

    let title_lbl = gtk::Label::new(Some(&event.title));
    title_lbl.add_css_class("heading");
    title_lbl.set_xalign(0.0);
    title_lbl.set_ellipsize(pango::EllipsizeMode::End);
    title_lbl.set_single_line_mode(true);

    text_box.append(&year_lbl);
    text_box.append(&title_lbl);
    outer.append(&text_box);

    let button = gtk::Button::new();
    button.set_child(Some(&outer));
    button.add_css_class("flat");
    button.add_css_class("event-button");
    button
}

fn show_event_detail(
    parent: &gtk::Window,
    title: &str,
    slug: &str,
    start: i64,
    end: i64,
    image: Option<String>,
) {
    let dialog = adw::Window::builder()
        .transient_for(parent)
        .modal(true)
        .default_width(480)
        .default_height(600)
        .build();

    let toolbar = adw::ToolbarView::new();
    toolbar.add_top_bar(&adw::HeaderBar::new());

    let content = gtk::Box::new(gtk::Orientation::Vertical, 12);
    content.set_margin_top(16);
    content.set_margin_bottom(16);
    content.set_margin_start(16);
    content.set_margin_end(16);

    if let Some(image_url) = image {
        let picture = gtk::Picture::new();
        picture.set_content_fit(gtk::ContentFit::Cover);
        picture.set_size_request(-1, 220);
        content.append(&picture);

        let picture_clone = picture.clone();
        glib::spawn_future_local(async move {
            let file = gio::File::for_uri(&image_url);
            if let Ok((bytes, _etag)) = file.load_contents_future().await {
                if let Ok(texture) = gdk::Texture::from_bytes(&glib::Bytes::from(&bytes)) {
                    picture_clone.set_paintable(Some(&texture));
                }
            }
        });
    }

    let years = gtk::Label::new(Some(&year_range_label(start, end)));
    years.add_css_class("dim-label");
    years.set_xalign(0.0);

    let title_lbl = gtk::Label::new(Some(title));
    title_lbl.add_css_class("title-1");
    title_lbl.set_xalign(0.0);
    title_lbl.set_wrap(true);

    let slug_lbl = gtk::Label::new(Some(&format!("Reference: {slug}")));
    slug_lbl.set_xalign(0.0);
    slug_lbl.set_wrap(true);

    content.append(&years);
    content.append(&title_lbl);
    content.append(&slug_lbl);

    let scroller = gtk::ScrolledWindow::new();
    scroller.set_child(Some(&content));

    toolbar.set_content(Some(&scroller));
    dialog.set_content(Some(&toolbar));
    dialog.present();
}
