use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use adw::prelude::*;
use gettextrs::gettext;
use gtk::prelude::*;
use relm4::prelude::*;
use xbible_engine::features::store::StoreItem;

#[derive(Debug, Clone, PartialEq)]
pub enum TranslationCardStatus {
    Idle,
    Pending,
    Installing { fraction: f64 },
    Installed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct TranslationCardInit {
    pub item: StoreItem,
    pub status: TranslationCardStatus,
    pub can_delete: bool,
    pub width: i32,
}

impl Default for TranslationCardInit {
    fn default() -> Self {
        Self {
            item: unreachable!("pass TranslationCardInit with StoreItem"),
            status: TranslationCardStatus::Idle,
            can_delete: false,
            width: 150,
        }
    }
}

#[derive(Debug)]
pub struct TranslationCard {
    item: StoreItem,
    status: TranslationCardStatus,
    can_delete: bool,
    width: i32,
}

#[derive(Debug)]
pub enum TranslationCardInput {
    SetStatus(TranslationCardStatus),
    SetCanDelete(bool),
    InstallClicked,
    OpenClicked,
    CardClicked,
    DeleteClicked,
    UpdateClicked,
}

#[derive(Debug, Clone)]
pub enum TranslationCardOutput {
    Install(StoreItem),
    Open(StoreItem),
    Selected(StoreItem),
    Delete(StoreItem),
    Update(StoreItem),
}

#[relm4::component(pub)]
impl Component for TranslationCard {
    type Init = TranslationCardInit;
    type Input = TranslationCardInput;
    type Output = TranslationCardOutput;
    type CommandOutput = ();

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 8,
            set_margin_all: 8,

            // ── Physical Book Cover ────────────────────────────────────────
            // Same construction as ModuleTile's book: a plain gtk::Box with
            // inline_css doing the gradient, base color, uneven rounded
            // corners, drop shadow and spine highlight. No DrawingArea.
            gtk::Box {
                set_size_request: (model.width, (model.width as f64 * 1.4).round() as i32),
                set_halign: gtk::Align::Start,
                set_overflow: gtk::Overflow::Hidden,

                #[watch]
                inline_css: &{
                    // StoreItem has no signature_color, so derive a stable
                    // (hue, sat, brightness) from the item id, then run it
                    // through the exact same HSL derivation ModuleTile uses.
                    let (h, s_b, b) = deterministic_hsb(&model.item.id);

                    let l = b * (1.0 - s_b / 2.0);
                    let s_l = if l == 0.0 || l == 1.0 {
                        0.0
                    } else {
                        (b - l) / l.min(1.0 - l)
                    };

                    let final_s = (s_l * 0.85).clamp(0.0, 1.0);
                    let final_l = (l * 0.90).clamp(0.0, 1.0);

                    let base_color = format!(
                        "hsl({}, {}%, {}%)",
                        (h * 360.0).round() as u16,
                        (final_s * 100.0).round() as u16,
                        (final_l * 100.0).round() as u16
                    );

                    format!(
                        "background-image: linear-gradient(to right, \
                            rgba(0, 0, 0, 0.40) 0%, \
                            rgba(255, 255, 255, 0.12) 6%, \
                            transparent 15%); \
                        background-color: {}; \
                        border-radius: 4px 12px 12px 4px; \
                        box-shadow: 5px 10px 20px rgba(0, 0, 0, 0.45); \
                        border-left: 3px solid rgba(255, 255, 255, 0.15);",
                        base_color
                    )
                },

                // Click gesture on the physical book.
                add_controller = gtk::GestureClick {
                    connect_released[sender] => move |_, _, _, _| {
                        sender.input(TranslationCardInput::CardClicked);
                    },
                },

                // Cover content: title centered, language pinned near bottom.
                // `osd` restores the floating-over-cover look (no black box).
                gtk::Box {
                    set_hexpand: true,
                    set_vexpand: true,
                    set_valign: gtk::Align::Fill,
                    set_halign: gtk::Align::Fill,
                    set_margin_start: 15,
                    set_margin_end: 15,
                    set_margin_top: 24,
                    set_margin_bottom: 24,
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 0,

                    gtk::Label {
                        set_label: &model.item.name,
                        set_wrap: true,
                        set_wrap_mode: gtk::pango::WrapMode::WordChar,
                        set_max_width_chars: 1,
                        set_justify: gtk::Justification::Center,
                        set_lines: 6,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_valign: gtk::Align::Center,
                        inline_css: "color: white; font-weight: 800; font-size: 1.05rem; text-shadow: 0 2px 4px rgba(0,0,0,0.8);",
                    },

                    // Spacer (equivalent to SwiftUI Spacer()).
                    gtk::Box { set_vexpand: true },

                    gtk::Label {
                        set_label: &model.language_upper(),
                        set_visible: !model.language_upper().is_empty(),
                        set_max_width_chars: 1,
                        set_halign: gtk::Align::Center,
                        set_valign: gtk::Align::End,
                        set_margin_top: 12,
                        inline_css: "color: rgba(255, 255, 255, 0.7); font-weight: 700; font-size: 0.75rem; letter-spacing: 1.2px;",
                    },
                },
            },

            // ── Bottom Row: Short Name + Badge + Action Control ────────────
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 6,
                set_size_request: (model.width, -1),
                set_halign: gtk::Align::Start,
                set_margin_top: 4,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 2,
                    set_hexpand: true,
                    set_valign: gtk::Align::Center,

                    gtk::Label {
                        set_label: &model.display_short_name(),
                        set_halign: gtk::Align::Start,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        inline_css: "font-weight: 700; font-size: 0.9rem; color: @window_fg_color;",
                    },

                    gtk::Label {
                        set_label: model.badge_text(),
                        set_visible: !model.badge_text().is_empty(),
                        set_halign: gtk::Align::Start,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        inline_css: "font-weight: 500; font-size: 0.8rem; opacity: 0.7; color: @window_fg_color;",
                    },
                },

                #[name = "action_container"]
                gtk::Box {
                    set_valign: gtk::Align::Center,
                    set_halign: gtk::Align::End,
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            item: init.item,
            status: init.status,
            can_delete: init.can_delete,
            width: init.width,
        };

        let widgets = view_output!();

        rebuild_action_widget(
            &widgets.action_container,
            &model.status,
            model.can_delete,
            &sender,
        );

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        widgets: &mut Self::Widgets,
        message: Self::Input,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            TranslationCardInput::SetStatus(status) => {
                self.status = status;
                rebuild_action_widget(
                    &widgets.action_container,
                    &self.status,
                    self.can_delete,
                    &sender,
                );
            }
            TranslationCardInput::SetCanDelete(can_delete) => {
                self.can_delete = can_delete;
                rebuild_action_widget(
                    &widgets.action_container,
                    &self.status,
                    self.can_delete,
                    &sender,
                );
            }
            TranslationCardInput::InstallClicked => {
                let _ = sender.output(TranslationCardOutput::Install(self.item.clone()));
            }
            TranslationCardInput::OpenClicked => {
                let _ = sender.output(TranslationCardOutput::Open(self.item.clone()));
            }
            TranslationCardInput::CardClicked => {
                let _ = sender.output(TranslationCardOutput::Selected(self.item.clone()));
            }
            TranslationCardInput::DeleteClicked => {
                let _ = sender.output(TranslationCardOutput::Delete(self.item.clone()));
            }
            TranslationCardInput::UpdateClicked => {
                let _ = sender.output(TranslationCardOutput::Update(self.item.clone()));
            }
        }
    }
}

impl TranslationCard {
    fn display_short_name(&self) -> String {
        if self.item.short_name.trim().is_empty() {
            self.item.name.clone()
        } else {
            self.item.short_name.clone()
        }
    }

    fn badge_text(&self) -> &str {
        self.item.badge.as_deref().unwrap_or("")
    }

    fn language_upper(&self) -> String {
        let lang = self
            .item
            .language_name
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(&self.item.language);
        lang.to_uppercase()
    }
}

/// Derives a stable `(hue, saturation, brightness)` triple from the item id
/// so the cover color is deterministic across rebuilds — same shape as
/// `SwordModule::signature_color`, just sourced from a hash instead.
fn deterministic_hsb(id: &str) -> (f64, f64, f64) {
    let mut hasher = DefaultHasher::new();
    id.hash(&mut hasher);
    let hash = hasher.finish();

    // Spread hue across the full circle, keep saturation/brightness in the
    // range ModuleTile's derivation expects (rich, readable behind white).
    let hue = (hash % 360) as f64 / 360.0;
    let saturation = 0.55 + ((hash >> 8) % 30) as f64 / 100.0; // 0.55..0.85
    let brightness = 0.45 + ((hash >> 16) % 25) as f64 / 100.0; // 0.45..0.70

    (hue, saturation, brightness)
}

fn rebuild_action_widget(
    container: &gtk::Box,
    status: &TranslationCardStatus,
    can_delete: bool,
    sender: &ComponentSender<TranslationCard>,
) {
    while let Some(child) = container.first_child() {
        container.remove(&child);
    }

    if can_delete {
        let menu = gtk::gio::Menu::new();
        menu.append(Some(&gettext("Open")), Some("card.open"));
        menu.append(Some(&gettext("Update")), Some("card.update"));
        menu.append(Some(&gettext("Delete")), Some("card.delete"));

        let actions = gtk::gio::SimpleActionGroup::new();

        let open_act = gtk::gio::SimpleAction::new("open", None);
        {
            let sender = sender.clone();
            open_act.connect_activate(move |_, _| {
                sender.input(TranslationCardInput::OpenClicked);
            });
        }
        actions.add_action(&open_act);

        let update_act = gtk::gio::SimpleAction::new("update", None);
        {
            let sender = sender.clone();
            update_act.connect_activate(move |_, _| {
                sender.input(TranslationCardInput::UpdateClicked);
            });
        }
        actions.add_action(&update_act);

        let delete_act = gtk::gio::SimpleAction::new("delete", None);
        {
            let sender = sender.clone();
            delete_act.connect_activate(move |_, _| {
                sender.input(TranslationCardInput::DeleteClicked);
            });
        }
        actions.add_action(&delete_act);

        container.insert_action_group("card", Some(&actions));

        let menu_btn = gtk::MenuButton::builder()
            .icon_name("view-more-horizontal-symbolic")
            .menu_model(&menu)
            .has_frame(false)
            .width_request(28)
            .height_request(28)
            .build();

        container.append(&menu_btn);
        return;
    }

    match status {
        TranslationCardStatus::Idle => {
            let btn = pill_button(&gettext("Get"));
            btn.connect_clicked({
                let sender = sender.clone();
                move |_| sender.input(TranslationCardInput::InstallClicked)
            });
            container.append(&btn);
        }
        TranslationCardStatus::Installed => {
            let btn = pill_button(&gettext("Open"));
            btn.connect_clicked({
                let sender = sender.clone();
                move |_| sender.input(TranslationCardInput::OpenClicked)
            });
            container.append(&btn);
        }
        TranslationCardStatus::Failed(_) => {
            let btn = pill_button(&gettext("Retry"));
            btn.connect_clicked({
                let sender = sender.clone();
                move |_| sender.input(TranslationCardInput::InstallClicked)
            });
            container.append(&btn);
        }
        TranslationCardStatus::Pending => {
            let spinner = gtk::Spinner::builder()
                .spinning(true)
                .width_request(24)
                .height_request(24)
                .build();
            container.append(&spinner);
        }
        TranslationCardStatus::Installing { fraction } => {
            let clamped = fraction.clamp(0.0, 1.0);
            let pct = (clamped * 100.0).round() as i32;

            let overlay = gtk::Overlay::new();
            overlay.set_size_request(32, 32);

            let bar = gtk::LevelBar::builder()
                .min_value(0.0)
                .max_value(1.0)
                .value(clamped)
                .width_request(32)
                .height_request(6)
                .valign(gtk::Align::End)
                .build();

            let pct_label = gtk::Label::builder()
                .label(&format!("{pct}%"))
                .valign(gtk::Align::Center)
                .halign(gtk::Align::Center)
                .build();
            pct_label.add_css_class("caption");
            pct_label.add_css_class("numeric");

            overlay.set_child(Some(&pct_label));
            overlay.add_overlay(&bar);
            container.append(&overlay);
        }
    }
}

fn pill_button(label: &str) -> gtk::Button {
    let btn = gtk::Button::with_label(label);
    btn.add_css_class("suggested-action");
    btn
}
