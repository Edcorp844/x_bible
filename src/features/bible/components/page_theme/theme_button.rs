use std::cell::Cell;
use std::rc::Rc;
use std::time::Instant;

use gtk::cairo;
use gtk::glib;
use gtk::prelude::*;
use relm4::RelmSetChildExt;
use relm4::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender};

use crate::features::bible::components::page::helpers::AvailableFonts;
use crate::features::bible::components::page::verse_components::verse::VerseInputMessage;
use crate::features::core::display_configurations::config::TextConfig;

// ---- Layout Constants ---------------------------------------------------
const COLLAPSED: f64 = 64.0;
const MENU_WIDTH: i32 = 340;
const SECTION_GAP: i32 = 8; // Reduced spacing between cards

const EXPAND_MS: f64 = 300.0;
const COLLAPSE_MS: f64 = 200.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MenuState {
    Collapsed,
    Animating,
    Expanded,
}

pub struct ExpandingThemeMenu {
    config: TextConfig,
    state: MenuState,
    progress: Rc<Cell<f64>>,
    hovered: Rc<Cell<bool>>,
}

#[derive(Debug)]
pub enum ThemeMenuInput {
    CanvasClicked { x: f64, y: f64 },
    CloseClicked,
    HoverChanged(bool),
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
}

#[derive(Debug)]
pub enum ThemeMenuOutput {
    OpenThemePopup,
    ToggleDisplay(VerseInputMessage),
}

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
        let hovered = Rc::new(Cell::new(false));

        let canvas = gtk::DrawingArea::new();
        canvas.set_width_request(COLLAPSED as i32);
        canvas.set_height_request(COLLAPSED as i32);
        canvas.set_halign(gtk::Align::End);
        canvas.set_valign(gtk::Align::End);

        {
            let progress = progress.clone();
            let hovered = hovered.clone();
            canvas.set_draw_func(move |_area, cr, w, h| {
                draw_collapsed_button(cr, w as f64, h as f64, progress.get(), hovered.get());
            });
        }

        let click = gtk::GestureClick::new();
        {
            let sender = sender.clone();
            click.connect_released(move |g, _n, x, y| {
                g.set_state(gtk::EventSequenceState::Claimed);
                sender.input(ThemeMenuInput::CanvasClicked { x, y });
            });
        }
        canvas.add_controller(click);

        let motion = gtk::EventControllerMotion::new();
        {
            let sender = sender.clone();
            motion.connect_enter(move |_, _, _| sender.input(ThemeMenuInput::HoverChanged(true)));
        }
        {
            let sender = sender.clone();
            motion.connect_leave(move |_| sender.input(ThemeMenuInput::HoverChanged(false)));
        }
        canvas.add_controller(motion);

        root.set_child(Some(&canvas));

        let content = build_content_box(&init, &sender);
        content.set_halign(gtk::Align::End);
        content.set_valign(gtk::Align::End);
        content.set_opacity(0.0);
        content.set_visible(false);
        content.set_can_target(false);
        root.add_overlay(&content);

        let model = Self {
            config: init,
            state: MenuState::Collapsed,
            progress,
            hovered,
        };

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
            ThemeMenuInput::HoverChanged(v) => {
                self.hovered.set(v);
                widgets.canvas.queue_draw();
            }

            ThemeMenuInput::CanvasClicked { .. } => {
                if self.state == MenuState::Collapsed {
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
                if !is_expanded {
                    widgets.canvas.set_visible(true);
                }
            }

            ThemeMenuInput::OpenThemePopupClicked => {
                let _ = sender.output(ThemeMenuOutput::OpenThemePopup);
            }

            ThemeMenuInput::FontSizeChanged(v) => {
                let _ = sender.output(ThemeMenuOutput::ToggleDisplay(
                    VerseInputMessage::ChangeFontSize(v),
                ));
            }

            ThemeMenuInput::FontFamilyChanged(_family) => {}

            ThemeMenuInput::ToggleChristWordsRed(active) => {
                let _ = sender.output(ThemeMenuOutput::ToggleDisplay(
                    VerseInputMessage::PutChristWordsInRed(active),
                ));
            }

            ThemeMenuInput::ToggleNotes(active) => {
                let msg = if active {
                    VerseInputMessage::EnableNotes
                } else {
                    VerseInputMessage::DisableNotes
                };
                let _ = sender.output(ThemeMenuOutput::ToggleDisplay(msg));
            }

            ThemeMenuInput::ToggleStrongs(_active) => {}
            ThemeMenuInput::ToggleLemma(_active) => {}
            ThemeMenuInput::ToggleMorph(_active) => {}
            ThemeMenuInput::AddedWordsStyleChanged(_style) => {}
        }

        let p = self.progress.get();
        widgets.content.set_opacity(p);

        widgets.canvas.set_visible(p < 0.99);
        widgets.canvas.queue_draw();
    }
}

// ---- Collapsed Button Indicator -------------------------------------------

fn draw_collapsed_button(cr: &cairo::Context, w: f64, h: f64, p: f64, hovered: bool) {
    if p >= 0.99 {
        return;
    }

    let alpha = if hovered { 0.7 } else { 0.5 };
    let radius = COLLAPSED / 2.0;

    cr.arc(w / 2.0, h / 2.0, radius, 0.0, std::f64::consts::TAU);
    cr.set_source_rgba(0.08, 0.09, 0.11, alpha * (1.0 - p));
    let _ = cr.fill_preserve();

    cr.set_source_rgba(1.0, 1.0, 1.0, 0.1 * (1.0 - p));
    cr.set_line_width(1.0);
    let _ = cr.stroke();

    let cx = w / 2.0;
    let cy = h / 2.0;
    cr.set_source_rgba(1.0, 1.0, 1.0, 1.0 - p);
    cr.set_line_width(2.0);
    for i in 0..3 {
        let yy = cy - 8.0 + i as f64 * 8.0;
        cr.move_to(cx - 10.0, yy);
        cr.line_to(cx + 10.0, yy);
    }
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

// ---- Layout Construction (Auto-Fitting Cards) ------------------------------

fn build_card_container() -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Vertical, 10);

    card.add_css_class("card");
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

    // Top Header Row with Title + Close Button
    let header_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);

    let title = gtk::Label::new(Some("Theme and Font"));
    title.add_css_class("title-2");
    title.set_hexpand(true);
    title.set_xalign(0.0);

    let close_btn = gtk::Button::from_icon_name("window-close-symbolic");
    //close_btn.add_css_class("flat");
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
    let small_a = gtk::Label::new(Some("A"));
    small_a.add_css_class("title-4");

    let big_a = gtk::Label::new(Some("A"));
    big_a.add_css_class("title-1");

    let scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 12.0, 32.0, 1.0);
    scale.set_hexpand(true);
    scale.add_css_class("accent");
    scale.set_value(config.read().unwrap().font_size());
    {
        let sender = sender.clone();
        scale.connect_value_changed(move |s| {
            sender.input(ThemeMenuInput::FontSizeChanged(s.value()));
        });
    }

    slider_box.append(&small_a);
    slider_box.append(&scale);
    slider_box.append(&big_a);
    top_card.append(&slider_box);


    //Fonts
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

    // Populate font widgets dynamically using your exact helper
    // BiblePage::populate_fonts_container(&fonts_box, sender.clone(), config.clone());
    while let Some(child) = fonts_box.first_child() {
        fonts_box.remove(&child);
    }

    for font in AvailableFonts::all() {
        //let widget = Self::font_menu_widget(font, sender.clone(), config.clone());
        let is_active = font == config.read().unwrap().font();
        let name = font.to_string();

        let font_button = gtk::Box::builder()
            .css_classes(vec!["menu-font-option", "clickable"])
            .cursor(&gtk::gdk::Cursor::from_name("pointer", None).unwrap())
            .build();

        if is_active {
            font_button.add_css_class("menu-font-option-active");
        }

        let markup = match font {
            AvailableFonts::System => format!("<span size='small'>{}</span>", name),
            _ => format!("<span face='{0}' size='small'>{0}</span>", name),
        };

        font_button.append(&gtk::Label::builder().use_markup(true).label(markup).build());

        let click = gtk::GestureClick::new();
        let font_clone = font.clone();
        let font_sender = sender.clone();
        click.connect_released(move |_, _, _, _| {
            let _ = font_sender.output(ThemeMenuOutput::ToggleDisplay(
                VerseInputMessage::ChangeFont(font_clone.clone()),
            ));
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
    book_options.add_css_class("title-2");
    book_options.set_xalign(0.0);
    bottom_card.append(&book_options);

    let text_header = gtk::Label::new(Some("Text"));
    text_header.add_css_class("title-4");
    text_header.set_xalign(0.0);
    bottom_card.append(&text_header);

    let christ_red = gtk::CheckButton::with_label("Words of Christ in Red");
    christ_red.set_active(config.read().unwrap().christ_words_red());
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

    let drop_down = gtk::DropDown::from_strings(&["Italic", "Underline", "None"]);
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
    notes.set_active(config.read().unwrap().show_notes());
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
