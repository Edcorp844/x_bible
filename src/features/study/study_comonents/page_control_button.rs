use std::cell::Cell;
use std::rc::Rc;
use std::time::Instant;

use gtk::cairo;
use gtk::glib;
use gtk::prelude::*;
use relm4::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender};

use crate::core::ui::text_config::{AddedWordsStyle, TextConfig};

// ---- Layout Constants ---------------------------------------------------
const PILL_WIDTH: f64 = 220.0;
const PILL_HEIGHT: f64 = 52.0;
const MENU_WIDTH: i32 = 340;
const SECTION_GAP: i32 = 8;

const EXPAND_MS: f64 = 300.0;
const COLLAPSE_MS: f64 = 200.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuState {
    Collapsed,
    Animating,
    Expanded,
}

#[derive(Debug)]
pub struct ExpandingThemeMenu {
    config: TextConfig,
    state: MenuState,
    progress: Rc<Cell<f64>>,
    hovered_zone: Rc<Cell<Option<usize>>>,
    pub has_prev: Rc<Cell<bool>>,
    pub has_next: Rc<Cell<bool>>,
    pub chapter_label: String,
}

#[derive(Debug)]
pub enum ThemeMenuInput {
    CanvasClicked {
        x: f64,
        y: f64,
    },
    CanvasMotion {
        x: f64,
        y: f64,
    },
    CanvasLeave,
    CloseClicked,
    AnimDone(MenuState),
    OpenThemePopupClicked,
    FontSizeChanged(f64),
    FontFamilyChanged(String),
    ToggleChristWordsRed(bool),
    ToggleNotes(bool),
    ToggleStrongs(bool),
    ToggleLemma(bool),
    ToggleMorph(bool),
    AddedWordsStyleChanged(String),
    SetNavigationState {
        has_prev: bool,
        has_next: bool,
        chapter_label: String,
    },
}

#[derive(Debug)]
pub enum ThemeMenuOutput {
    OpenThemePopup,
    DimBackground(bool),
    PreviousChapter,
    NextChapter,
}

#[derive(Debug)]
pub struct ThemeMenuWidgets {
    canvas: gtk::DrawingArea,
    content: gtk::Box,
}

impl Component for ExpandingThemeMenu {
    type Init = TextConfig;
    type Input = ThemeMenuInput;
    type Output = ThemeMenuOutput;
    type CommandOutput = ();
    type Root = gtk::Overlay;
    type Widgets = ThemeMenuWidgets;

    fn init_root() -> Self::Root {
        gtk::Overlay::builder()
            .halign(gtk::Align::End)
            .valign(gtk::Align::End)
            .build()
    }

    fn init(
        init: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let progress = Rc::new(Cell::new(0.0_f64));
        let hovered_zone = Rc::new(Cell::new(None));
        let has_prev = Rc::new(Cell::new(false));
        let has_next = Rc::new(Cell::new(false));

        let model = Self {
            config: init.clone(),
            state: MenuState::Collapsed,
            progress: progress.clone(),
            hovered_zone: hovered_zone.clone(),
            has_prev: has_prev.clone(),
            has_next: has_next.clone(),
            chapter_label: String::new(),
        };

        let canvas = gtk::DrawingArea::new();
        canvas.set_width_request(PILL_WIDTH as i32);
        canvas.set_height_request(PILL_HEIGHT as i32);
        canvas.set_hexpand(false);
        canvas.set_vexpand(false);
        canvas.set_halign(gtk::Align::End);
        canvas.set_valign(gtk::Align::End);
        canvas.set_can_target(true);
        canvas.set_overflow(gtk::Overflow::Hidden);
        canvas.set_margin_end(16);
        canvas.set_margin_bottom(16);

        {
            let progress = progress.clone();
            let hovered_zone = hovered_zone.clone();
            let draw_has_prev = has_prev.clone();
            let draw_has_next = has_next.clone();
            canvas.set_draw_func(move |_area, cr, w, h| {
                // Read model dynamic states implicitly via closure scope
                draw_pill_toolbar(
                    cr,
                    w as f64,
                    h as f64,
                    progress.get(),
                    hovered_zone.get(),
                    draw_has_prev.get(),
                    draw_has_next.get(),
                    "",
                );
            });
        }

        let click = gtk::GestureClick::new();
        {
            let gesture_sender = sender.clone();
            click.connect_released(move |g, _n, x, y| {
                g.set_state(gtk::EventSequenceState::Claimed);
                gesture_sender.input(ThemeMenuInput::CanvasClicked { x, y });
            });
        }
        canvas.add_controller(click);

        let motion = gtk::EventControllerMotion::new();
        {
            let event_sender = sender.clone();
            motion.connect_motion(move |_, x, y| {
                event_sender.input(ThemeMenuInput::CanvasMotion { x, y })
            });
            let canvas_sender = sender.clone();
            motion.connect_leave(move |_| canvas_sender.input(ThemeMenuInput::CanvasLeave));
        }
        canvas.add_controller(motion);

        root.set_child(Some(&canvas));

        let content = build_content_box(&init, &sender);
        content.set_halign(gtk::Align::End);
        content.set_valign(gtk::Align::End);
        content.set_margin_end(16);
        content.set_margin_bottom(16);
        content.set_opacity(0.0);
        content.set_visible(false);
        content.set_can_target(false);
        root.add_overlay(&content);

        let widgets = ThemeMenuWidgets { canvas, content };
        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            ThemeMenuInput::CanvasMotion { x, y: _ } => {
                let zone_w = PILL_WIDTH / 3.0;
                let zone = if x < zone_w {
                    if self.has_prev.get() { Some(0) } else { None }
                } else if x < zone_w * 2.0 {
                    Some(1)
                } else {
                    if self.has_next.get() { Some(2) } else { None }
                };
                self.hovered_zone.set(zone);
                widgets.canvas.queue_draw();
            }

            ThemeMenuInput::CanvasLeave => {
                self.hovered_zone.set(None);
                widgets.canvas.queue_draw();
            }

            ThemeMenuInput::CanvasClicked { x, y: _ } => {
                if self.state == MenuState::Collapsed {
                    let zone_w = PILL_WIDTH / 3.0;
                    if x < zone_w {
                        if self.has_prev.get() {
                            let _ = sender.output(ThemeMenuOutput::PreviousChapter);
                        }
                    } else if x > zone_w * 2.0 {
                        if self.has_next.get() {
                            let _ = sender.output(ThemeMenuOutput::NextChapter);
                        }
                    } else {
                        // Center Button: Expand Menu
                        self.state = MenuState::Animating;
                        widgets.content.set_visible(true);

                        animate(
                            &widgets.canvas,
                            &self.progress,
                            1.0,
                            EXPAND_MS,
                            ease_out_back,
                            sender.clone(),
                            MenuState::Expanded,
                        );
                    }
                }
            }

            ThemeMenuInput::CloseClicked => {
                if self.state == MenuState::Expanded {
                    self.state = MenuState::Animating;

                    animate(
                        &widgets.canvas,
                        &self.progress,
                        0.0,
                        COLLAPSE_MS,
                        ease_in,
                        sender.clone(),
                        MenuState::Collapsed,
                    );
                }
            }

            ThemeMenuInput::AnimDone(final_state) => {
                self.state = final_state;
                let is_expanded = final_state == MenuState::Expanded;
                widgets.content.set_visible(is_expanded);
                widgets.content.set_can_target(is_expanded);
                let _ = sender.output(ThemeMenuOutput::DimBackground(is_expanded));
                if !is_expanded {
                    widgets.canvas.set_visible(true);
                }
            }

            ThemeMenuInput::OpenThemePopupClicked => {
                let _ = sender.output(ThemeMenuOutput::OpenThemePopup);
            }

            ThemeMenuInput::FontSizeChanged(value) => {
                self.config.font_size_percent = ((value / 17.0) * 100.0).round() as u32;
            }

            ThemeMenuInput::FontFamilyChanged(family) => {
                self.config.use_system_font = false;
                self.config.font_family_name = family;
            }

            ThemeMenuInput::ToggleChristWordsRed(active) => {
                self.config.show_red_words = active;
            }

            ThemeMenuInput::ToggleNotes(active) => {
                self.config.show_verse_notes = active;
            }

            ThemeMenuInput::SetNavigationState {
                has_prev,
                has_next,
                chapter_label,
            } => {
                self.has_prev.set(has_prev);
                self.has_next.set(has_next);
                self.chapter_label = chapter_label.clone();
                widgets.canvas.set_tooltip_text(Some(&chapter_label));
                widgets.canvas.queue_draw();
            }

            ThemeMenuInput::ToggleStrongs(active) => {
                self.config.show_strongs = active;
            }
            ThemeMenuInput::ToggleLemma(active) => {
                self.config.show_lemma = active;
            }
            ThemeMenuInput::ToggleMorph(active) => {
                self.config.show_morph = active;
            }
            ThemeMenuInput::AddedWordsStyleChanged(style) => {
                if let Some(value) = AddedWordsStyle::ALL
                    .iter()
                    .find(|item| item.display_name() == style)
                {
                    self.config.added_words_style = *value;
                }
            }
        }

        let p = self.progress.get();
        widgets.content.set_opacity(p);

        widgets.canvas.set_visible(p < 0.99);

        // Update custom dynamic draw callback with current sensitivity states
        let progress = self.progress.clone();
        let hovered_zone = self.hovered_zone.clone();
        let has_prev = self.has_prev.clone();
        let has_next = self.has_next.clone();

        widgets.canvas.set_draw_func(move |_area, cr, w, h| {
            draw_pill_toolbar(
                cr,
                w as f64,
                h as f64,
                progress.get(),
                hovered_zone.get(),
                has_prev.get(),
                has_next.get(),
                "",
            );
        });

        widgets.canvas.queue_draw();
    }
}

// ---- Pill Toolbar Renderer --------------------------------------------------

fn draw_pill_toolbar(
    cr: &cairo::Context,
    w: f64,
    h: f64,
    p: f64,
    hovered_zone: Option<usize>,
    has_prev: bool,
    has_next: bool,
    chapter_label: &str,
) {
    if p >= 0.99 {
        return;
    }

    let alpha = 1.0 - p;
    let radius = h / 2.0;

    // Draw Pill Background Shape
    cr.new_sub_path();
    cr.arc(
        w - radius,
        radius,
        radius,
        -std::f64::consts::FRAC_PI_2,
        std::f64::consts::FRAC_PI_2,
    );
    cr.arc(
        radius,
        radius,
        radius,
        std::f64::consts::FRAC_PI_2,
        3.0 * std::f64::consts::FRAC_PI_2,
    );
    cr.close_path();

    cr.set_source_rgba(0.08, 0.09, 0.11, 0.85 * alpha);
    let _ = cr.fill_preserve();

    cr.set_source_rgba(1.0, 1.0, 1.0, 0.12 * alpha);
    cr.set_line_width(1.0);
    let _ = cr.stroke();

    // Hover Highlight Overlays
    if let Some(zone) = hovered_zone {
        let zone_w = w / 3.0;

        cr.set_source_rgba(1.0, 1.0, 1.0, 0.08 * alpha);
        match zone {
            0 if has_prev => {
                // Left Zone Highlight
                cr.arc(
                    radius,
                    radius,
                    radius,
                    std::f64::consts::FRAC_PI_2,
                    3.0 * std::f64::consts::FRAC_PI_2,
                );
                cr.line_to(zone_w, 0.0);
                cr.line_to(zone_w, h);
                cr.close_path();
                let _ = cr.fill();
            }
            1 => {
                // Middle Zone Highlight
                cr.rectangle(zone_w, 0.0, zone_w, h);
                let _ = cr.fill();
            }
            2 if has_next => {
                // Right Zone Highlight
                cr.move_to(w - zone_w, 0.0);
                cr.arc(
                    w - radius,
                    radius,
                    radius,
                    -std::f64::consts::FRAC_PI_2,
                    std::f64::consts::FRAC_PI_2,
                );
                cr.line_to(w - zone_w, h);
                cr.close_path();
                let _ = cr.fill();
            }
            _ => {}
        }
    }

    // Dividers
    cr.set_source_rgba(1.0, 1.0, 1.0, 0.15 * alpha);
    cr.set_line_width(1.0);

    cr.move_to(w / 3.0, 10.0);
    cr.line_to(w / 3.0, h - 10.0);

    cr.move_to(2.0 * w / 3.0, 10.0);
    cr.line_to(2.0 * w / 3.0, h - 10.0);
    let _ = cr.stroke();

    // Icons Setup
    cr.set_line_width(2.0);
    let _ = chapter_label;

    // 1. Left Arrow (<)
    let left_cx = w / 6.0;
    let cy = h / 2.0;
    let left_alpha = if has_prev { alpha } else { alpha * 0.25 };
    cr.set_source_rgba(1.0, 1.0, 1.0, left_alpha);

    cr.move_to(left_cx + 3.0, cy - 6.0);
    cr.line_to(left_cx - 3.0, cy);
    cr.line_to(left_cx + 3.0, cy + 6.0);
    let _ = cr.stroke();

    // 2. Middle Icon (Slats/Menu - Left aligned with shorter middle line)
    cr.set_source_rgba(1.0, 1.0, 1.0, alpha);
    let mid_cx = w / 2.0;
    cr.set_line_cap(cairo::LineCap::Round);

    for i in 0..3 {
        let yy = cy - 6.0 + i as f64 * 6.0;
        let start_x = mid_cx - 7.0;
        let end_x = if i == 1 { mid_cx + 2.0 } else { mid_cx + 7.0 };

        cr.move_to(start_x, yy);
        cr.line_to(end_x, yy);
    }
    let _ = cr.stroke();

    // 3. Right Arrow (>)

    let right_cx = 5.0 * w / 6.0;
    let right_alpha = if has_next { alpha } else { alpha * 0.25 };
    cr.set_source_rgba(1.0, 1.0, 1.0, right_alpha);

    cr.move_to(right_cx - 3.0, cy - 6.0);
    cr.line_to(right_cx + 3.0, cy);
    cr.line_to(right_cx - 3.0, cy + 6.0);
    let _ = cr.stroke();
}

fn animate(
    canvas: &gtk::DrawingArea,
    progress: &Rc<Cell<f64>>,
    target: f64,
    duration_ms: f64,
    ease: fn(f64) -> f64,
    sender: ComponentSender<ExpandingThemeMenu>,
    final_state: MenuState,
) {
    let start = progress.get();
    let start_time = Instant::now();
    let progress = progress.clone();

    canvas.add_tick_callback(move |area, _clock| {
        let t = (start_time.elapsed().as_secs_f64() * 1000.0 / duration_ms).min(1.0);
        let eased = ease(t);
        progress.set(start + (target - start) * eased);
        area.queue_draw();

        if t >= 1.0 {
            sender.input(ThemeMenuInput::AnimDone(final_state));
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
}

fn ease_out_back(t: f64) -> f64 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

fn ease_in(t: f64) -> f64 {
    t * t
}

// ---- Layout Construction (Content Overlay) ----------------------------------

fn build_card_container() -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 10);

    card.add_css_class("card");
    card.add_css_class("osd");
    card.add_css_class("menu-card-surface");

    let provider = gtk::CssProvider::new();
    provider.load_from_data(
        "
        .menu-card-surface {
            background-color: rgba(20, 22, 26, 0.92);
            border: 1px solid rgba(255, 255, 255, 0.08);
            border-radius: 20px;
            padding: 16px;
        }
        .fonts-scroll {
            background-color: transparent;
        }
        .close-btn {
            border-radius: 99px;
            padding: 4px;
        }
        ",
    );

    if let Some(display) = gtk::gdk::Display::default() {
        gtk::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    card
}

fn build_content_box(
    config: &TextConfig,
    sender: &ComponentSender<ExpandingThemeMenu>,
) -> gtk::Box {
    let outer = gtk::Box::new(gtk::Orientation::Vertical, SECTION_GAP);
    outer.set_width_request(MENU_WIDTH);

    // ==========================================
    // TOP CARD: Theme & Font Options
    // ==========================================
    let top_card = build_card_container();

    let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);

    let title = gtk::Label::new(Some("Theme and Font"));
    title.add_css_class("title-3");
    title.set_hexpand(true);
    title.set_xalign(0.0);

    let close_btn = gtk::Button::from_icon_name("window-close-symbolic");
    close_btn.add_css_class("close-btn");
    {
        let sender = sender.clone();
        close_btn.connect_clicked(move |_| {
            sender.input(ThemeMenuInput::CloseClicked);
        });
    }

    header_box.append(&title);
    header_box.append(&close_btn);
    top_card.append(&header_box);

    let font_label = gtk::Label::new(Some("Font Size"));
    font_label.add_css_class("title-4");
    font_label.set_xalign(0.0);
    top_card.append(&font_label);

    let slider_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let small_a = gtk::Image::from_icon_name("font-letter-symbolic");

    let big_a = gtk::Image::from_icon_name("font-letter-symbolic");
    big_a.set_pixel_size(30);

    let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 12.0, 32.0, 1.0);
    scale.set_hexpand(true);
    scale.set_value(17.0 * config.font_size_percent as f64 / 100.0);
    {
        let sender = sender.clone();
        scale.connect_value_changed(move |scale| {
            sender.input(ThemeMenuInput::FontSizeChanged(scale.value()));
        });
    }

    slider_box.append(&small_a);
    slider_box.append(&scale);
    slider_box.append(&big_a);
    top_card.append(&slider_box);

    let fonts_header = gtk::Label::new(Some("Fonts"));
    fonts_header.add_css_class("title-4");
    fonts_header.set_xalign(0.0);
    top_card.append(&fonts_header);

    let fonts_scroll = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::External)
        .vscrollbar_policy(gtk::PolicyType::Never)
        .overlay_scrolling(true)
        .build();
    fonts_scroll.add_css_class("fonts-scroll");

    let fonts_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);

    for name in ["Sans", "Serif", "Monospace", "Rounded"] {
        let is_active = config.resolved_font_family() == name;

        let font_button = gtk::Box::builder()
            .css_classes(vec!["menu-font-option", "clickable"])
            .cursor(&gtk::gdk::Cursor::from_name("pointer", None).unwrap())
            .build();

        if is_active {
            font_button.add_css_class("menu-font-option-active");
        }

        let markup = format!("<span face='{0}' size='small'>{0}</span>", name);

        font_button.append(&gtk::Label::builder().use_markup(true).label(markup).build());

        let click = gtk::GestureClick::new();
        let font_sender = sender.clone();
        let font_name = name.to_string();
        click.connect_released(move |_, _, _, _| {
            font_sender.input(ThemeMenuInput::FontFamilyChanged(font_name.clone()));
        });

        font_button.add_controller(click);
        fonts_box.append(&font_button);
    }

    fonts_scroll.set_child(Some(&fonts_box));
    top_card.append(&fonts_scroll);

    let customize = gtk::Button::new();
    let content = adw::ButtonContent::builder()
        .icon_name("emblem-system-symbolic")
        .label("Customize")
        .build();
    customize.set_child(Some(&content));
    {
        let sender = sender.clone();
        customize.connect_clicked(move |_| {
            sender.input(ThemeMenuInput::OpenThemePopupClicked);
        });
    }
    top_card.append(&customize);

    // ==========================================
    // BOTTOM CARD: Book Options & Lexicons
    // ==========================================
    let bottom_card = build_card_container();

    let book_options = gtk::Label::new(Some("Book Options"));
    book_options.add_css_class("title-3");
    book_options.set_xalign(0.0);
    bottom_card.append(&book_options);

    let text_header = gtk::Label::new(Some("Text"));
    text_header.add_css_class("title-4");
    text_header.set_xalign(0.0);
    bottom_card.append(&text_header);

    let christ_red = gtk::CheckButton::with_label("Words of Christ in Red");
    christ_red.set_active(config.show_red_words);
    {
        let sender = sender.clone();
        christ_red.connect_toggled(move |b| {
            sender.input(ThemeMenuInput::ToggleChristWordsRed(b.is_active()));
        });
    }
    bottom_card.append(&christ_red);

    let added_words_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let added_words_label = gtk::Label::new(Some("Added words"));
    added_words_label.set_hexpand(true);
    added_words_label.set_xalign(0.0);

    let all_styles = AddedWordsStyle::ALL;

    let strings: Vec<String> = all_styles.iter().map(|s| s.display_name()).collect();

    let str_slices: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();

    let drop_down = gtk::DropDown::from_strings(&str_slices);

    // Select current config option on startup
    let current_style = config.added_words_style;
    if let Some(initial_index) = all_styles.iter().position(|s| *s == current_style) {
        drop_down.set_selected(initial_index as u32);
    }

    {
        let sender = sender.clone();
        drop_down.connect_selected_notify(move |dd| {
            if let Some(item) = dd.selected_item() {
                if let Ok(string_obj) = item.downcast::<gtk::StringObject>() {
                    sender.input(ThemeMenuInput::AddedWordsStyleChanged(
                        string_obj.string().to_string(),
                    ));
                }
            }
        });
    }

    added_words_row.append(&added_words_label);
    added_words_row.append(&drop_down);
    bottom_card.append(&added_words_row);

    let lexicons_header = gtk::Label::new(Some("Lexicons"));
    lexicons_header.add_css_class("title-4");
    lexicons_header.set_xalign(0.0);
    bottom_card.append(&lexicons_header);

    let lex_grid = gtk::Grid::new();
    lex_grid.set_column_spacing(16);
    lex_grid.set_row_spacing(8);

    let strongs = gtk::CheckButton::with_label("Strongs");
    {
        let sender = sender.clone();
        strongs.connect_toggled(move |b| {
            sender.input(ThemeMenuInput::ToggleStrongs(b.is_active()));
        });
    }
    lex_grid.attach(&strongs, 0, 0, 1, 1);

    let lemma = gtk::CheckButton::with_label("Lemma");
    {
        let sender = sender.clone();
        lemma.connect_toggled(move |b| {
            sender.input(ThemeMenuInput::ToggleLemma(b.is_active()));
        });
    }
    lex_grid.attach(&lemma, 1, 0, 1, 1);

    let morph = gtk::CheckButton::with_label("Morph");
    {
        let sender = sender.clone();
        morph.connect_toggled(move |b| {
            sender.input(ThemeMenuInput::ToggleMorph(b.is_active()));
        });
    }
    lex_grid.attach(&morph, 0, 1, 1, 1);
    bottom_card.append(&lex_grid);

    let notes = gtk::CheckButton::with_label("Show verse Notes");
    notes.set_active(config.show_verse_notes);
    {
        let sender = sender.clone();
        notes.connect_toggled(move |b| {
            sender.input(ThemeMenuInput::ToggleNotes(b.is_active()));
        });
    }
    bottom_card.append(&notes);

    outer.append(&top_card);
    outer.append(&bottom_card);

    outer
}
