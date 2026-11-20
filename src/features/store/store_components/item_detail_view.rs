use adw::prelude::*;
use gettextrs::gettext;
use relm4::prelude::*;
use xbible_engine::features::store::{StoreItem, StoreShelf};

use crate::{
    features::store::store_item_ui_extension::StoreItemUIExt, workers::store_worker::InstallState,
};

#[derive(Debug, Clone)]
pub struct StoreItemDetailInit {
    pub item: StoreItem,
    pub shelves: Vec<StoreShelf>,
    pub active_install: Option<InstallState>,
}

#[derive(Debug)]
pub struct StoreItemDetailView {
    base_item: StoreItem,
    shelves: Vec<StoreShelf>,
    active_install: Option<InstallState>,
    is_description_expanded: bool,
    is_uninstalling: bool,
}

#[derive(Debug)]
pub enum StoreItemDetailInput {
    SyncState {
        shelves: Vec<StoreShelf>,
        active_install: Option<InstallState>,
    },
    ToggleDescription,
    InstallClicked,
    OpenClicked,
    RequestUninstall,
    ConfirmUninstall,
}

#[derive(Debug, Clone)]
pub enum StoreItemDetailOutput {
    InstallItem(StoreItem),
    OpenItem(StoreItem),
    UninstallItem(StoreItem),
}

#[relm4::component(pub)]
impl Component for StoreItemDetailView {
    type Init = StoreItemDetailInit;
    type Input = StoreItemDetailInput;
    type Output = StoreItemDetailOutput;
    type CommandOutput = ();

    view! {
        adw::NavigationPage {
            set_title: &model.current_item().name,

            #[wrap(Some)]
            set_child = &adw::ToolbarView {
                add_top_bar = &adw::HeaderBar {},

                #[wrap(Some)]
                set_content = &gtk::ScrolledWindow {
                    set_hscrollbar_policy: gtk::PolicyType::Never,
                    set_vexpand: true,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 20,

                        // ── Hero Section (Gradient + Book Cover + Title + Action) ──
                        #[name = "hero_box"]
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 12,
                            set_margin_top: 24,
                            set_margin_bottom: 12,
                            set_halign: gtk::Align::Fill,

                            #[name = "hero_bg_area"]
                            gtk::DrawingArea {
                                set_hexpand: true,
                                set_vexpand: true,
                            },

                            // Book Cover Artwork (150 x 210)
                            #[name = "cover_frame"]
                            gtk::Frame {
                                add_css_class: "card",
                                set_halign: gtk::Align::Center,
                                set_size_request: (150, 210),

                                gtk::Overlay {
                                    #[name = "cover_draw_area"]
                                    gtk::DrawingArea {
                                        set_hexpand: true,
                                        set_vexpand: true,
                                    },

                                    add_overlay = &gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,
                                        set_spacing: 12,
                                        set_margin_top: 24,
                                        set_margin_bottom: 24,
                                        set_margin_start: 15,
                                        set_margin_end: 15,

                                        #[name = "cover_title"]
                                        gtk::Label {
                                            set_label: &model.current_item().name,
                                            set_wrap: true,
                                            set_justify: gtk::Justification::Center,
                                            set_lines: 6,
                                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                                            set_vexpand: true,
                                            set_valign: gtk::Align::Center,
                                            add_css_class: "title-4",
                                            add_css_class: "osd",
                                        },

                                        #[name = "cover_lang"]
                                        gtk::Label {
                                            set_label: &model.current_item().language_display().to_uppercase(),
                                            set_visible: !&model.current_item().language_display().is_empty(),
                                            set_halign: gtk::Align::Center,
                                            add_css_class: "caption",
                                            add_css_class: "dim-label",
                                            add_css_class: "osd",
                                        },
                                    },
                                },
                            },

                            // Title + Language / Kind + Primary Action / Uninstall
                            #[name = "item_title"]
                            gtk::Label {
                                set_label: &model.current_item().name,
                                set_wrap: true,
                                set_justify: gtk::Justification::Center,
                                set_margin_start: 24,
                                set_margin_end: 24,
                                set_selectable: true,
                                add_css_class: "title-2",
                            },

                            #[name = "subtitle_row"]
                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 6,
                                set_halign: gtk::Align::Center,

                                gtk::Image {
                                    set_icon_name: Some("globe-symbolic"),
                                },

                                #[name = "lang_kind_label"]
                                gtk::Label {
                                    set_label: &model.language_and_kind_label(),
                                    add_css_class: "caption",
                                    add_css_class: "dim-label",
                                },
                            },

                            #[name = "action_box"]
                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 10,
                                set_halign: gtk::Align::Center,
                                set_margin_top: 4,
                                set_margin_bottom: 12,
                            },
                        },

                        // ── Capabilities Pills (Strongs / Audio) ──
                        #[name = "capabilities_box"]
                        gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_spacing: 10,
                            set_halign: gtk::Align::Center,
                            set_margin_start: 24,
                            set_margin_end: 24,
                        },

                        // ── Description Section ──
                        #[name = "description_box"]
                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 8,
                            set_margin_start: 24,
                            set_margin_end: 24,

                            gtk::Label {
                                set_label: &gettext("About"),
                                set_halign: gtk::Align::Start,
                                add_css_class: "heading",
                            },

                            #[name = "description_label"]
                            gtk::Label {
                                set_label: &model.description_text(),
                                set_wrap: true,
                                set_halign: gtk::Align::Start,
                                set_selectable: true,
                                set_lines: 5,
                                set_ellipsize: gtk::pango::EllipsizeMode::End,
                            },

                            #[name = "toggle_desc_btn"]
                            gtk::Button {
                                set_label: &gettext("More"),
                                set_halign: gtk::Align::End,
                                add_css_class: "flat",
                                connect_clicked[sender] => move |_| {
                                    sender.input(StoreItemDetailInput::ToggleDescription);
                                },
                            },
                        },

                        // ── Metadata Group (adw::PreferencesGroup) ──
                        #[name = "details_group"]
                        adw::PreferencesGroup {
                            set_title: &gettext("Details"),
                            set_margin_start: 24,
                            set_margin_end: 24,
                            set_margin_bottom: 32,
                        },
                    },
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
            base_item: init.item,
            shelves: init.shelves,
            active_install: init.active_install,
            is_description_expanded: false,
            is_uninstalling: false,
        };

        let widgets = view_output!();

        let item_clone = model.base_item.clone();
        widgets.cover_draw_area.set_draw_func({
            let item = item_clone.clone();
            move |_area, cr, w, h| {
                draw_cover_surface(cr, w as f64, h as f64, &item);
            }
        });

        model.refresh_dynamic_widgets(&widgets, &sender);

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
            StoreItemDetailInput::SyncState {
                shelves,
                active_install,
            } => {
                self.shelves = shelves;
                self.active_install = active_install;
                self.refresh_dynamic_widgets(widgets, &sender);
            }
            StoreItemDetailInput::ToggleDescription => {
                self.is_description_expanded = !self.is_description_expanded;
                if self.is_description_expanded {
                    widgets.description_label.set_lines(-1);
                    widgets
                        .description_label
                        .set_ellipsize(gtk::pango::EllipsizeMode::None);
                    widgets.toggle_desc_btn.set_label(&gettext("Less"));
                } else {
                    widgets.description_label.set_lines(5);
                    widgets
                        .description_label
                        .set_ellipsize(gtk::pango::EllipsizeMode::End);
                    widgets.toggle_desc_btn.set_label(&gettext("More"));
                }
            }
            StoreItemDetailInput::InstallClicked => {
                let _ = sender.output(StoreItemDetailOutput::InstallItem(self.current_item()));
            }
            StoreItemDetailInput::OpenClicked => {
                let _ = sender.output(StoreItemDetailOutput::OpenItem(self.current_item()));
            }
            StoreItemDetailInput::RequestUninstall => {
                let item = self.current_item();
                let dialog = adw::AlertDialog::new(
                    Some(&gettext("Remove {}?").replace("{}", &item.name)),
                    Some(
                        &gettext("This will delete downloaded data for {} from this device.")
                            .replace("{}", &item.name),
                    ),
                );
                dialog.add_response("cancel", &gettext("Cancel"));
                dialog.add_response("remove", &gettext("Remove"));
                dialog.set_response_appearance("remove", adw::ResponseAppearance::Destructive);
                dialog.set_default_response(Some("cancel"));
                dialog.set_close_response("cancel");

                let sender = sender.clone();
                dialog.connect_response(None, move |_dlg, response| {
                    if response == "remove" {
                        sender.input(StoreItemDetailInput::ConfirmUninstall);
                    }
                });
                dialog.present(Some(root));
            }
            StoreItemDetailInput::ConfirmUninstall => {
                self.is_uninstalling = true;
                self.refresh_dynamic_widgets(widgets, &sender);
                let _ = sender.output(StoreItemDetailOutput::UninstallItem(self.current_item()));
            }
        }
    }
}

impl StoreItemDetailView {
    fn current_item(&self) -> StoreItem {
        for shelf in &self.shelves {
            if let Some(found) = shelf.items.iter().find(|i| i.id == self.base_item.id) {
                return found.clone();
            }
        }
        self.base_item.clone()
    }

    fn active_install_for_item(&self) -> Option<&InstallState> {
        let item = self.current_item();
        self.active_install
            .as_ref()
            .filter(|inst| inst.item_id == item.id)
    }

    fn language_and_kind_label(&self) -> String {
        let item = self.current_item();
        format!("{} • {}", &item.language_display(), item.kind)
    }

    fn description_text(&self) -> String {
        let item = self.current_item();
        if item.description.trim().is_empty() {
            gettext("No description available for this item.")
        } else {
            item.description
        }
    }

    fn refresh_dynamic_widgets(
        &self,
        widgets: &<Self as Component>::Widgets,
        sender: &ComponentSender<Self>,
    ) {
        let item = self.current_item();

        widgets.item_title.set_label(&item.name);
        widgets.cover_title.set_label(&item.name);
        widgets
            .lang_kind_label
            .set_label(&self.language_and_kind_label());

        let has_long_desc = !item.description.trim().is_empty() && item.description.len() > 220;
        widgets.toggle_desc_btn.set_visible(has_long_desc);

        // Rebuild action buttons
        while let Some(child) = widgets.action_box.first_child() {
            widgets.action_box.remove(&child);
        }

        if let Some(install) = self.active_install_for_item()
            && install.failed.is_none()
            && !install.done
        {
            let clamped = install.fraction.clamp(0.0, 1.0);
            let is_installing =
                install.message.to_lowercase().contains("install") || clamped >= 0.85;
            let state_label = if is_installing {
                gettext("Installing")
            } else {
                gettext("Downloading")
            };

            let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);
            row.add_css_class("pill");
            row.add_css_class("suggested-action");

            let spinner = gtk::Spinner::builder().spinning(true).build();
            let lbl = gtk::Label::new(Some(&state_label));
            row.append(&spinner);
            row.append(&lbl);
            widgets.action_box.append(&row);
        } else if item.installed {
            let open_btn = gtk::Button::with_label(&gettext("Open"));
            open_btn.add_css_class("pill");
            open_btn.add_css_class("suggested-action");
            open_btn.connect_clicked({
                let sender = sender.clone();
                move |_| sender.input(StoreItemDetailInput::OpenClicked)
            });
            widgets.action_box.append(&open_btn);

            let remove_label = if self.is_uninstalling {
                gettext("Removing…")
            } else {
                gettext("Remove")
            };
            let remove_btn = gtk::Button::with_label(&remove_label);
            remove_btn.add_css_class("pill");
            remove_btn.add_css_class("destructive-action");
            remove_btn.set_sensitive(!self.is_uninstalling);
            remove_btn.connect_clicked({
                let sender = sender.clone();
                move |_| sender.input(StoreItemDetailInput::RequestUninstall)
            });
            widgets.action_box.append(&remove_btn);
        } else {
            let get_btn = gtk::Button::with_label(&gettext("Get"));
            get_btn.add_css_class("pill");
            get_btn.add_css_class("suggested-action");
            get_btn.set_sensitive(self.active_install.is_none());
            get_btn.connect_clicked({
                let sender = sender.clone();
                move |_| sender.input(StoreItemDetailInput::InstallClicked)
            });
            widgets.action_box.append(&get_btn);
        }

        // Rebuild capability pills
        while let Some(child) = widgets.capabilities_box.first_child() {
            widgets.capabilities_box.remove(&child);
        }
        widgets.capabilities_box.append(&capability_badge(
            &gettext("Strong's"),
            "accessories-dictionary-symbolic",
            item.has_strongs,
        ));
        widgets.capabilities_box.append(&capability_badge(
            &gettext("Audio"),
            "audio-speakers-symbolic",
            item.has_audio,
        ));

        // Rebuild metadata rows
        while let Some(child) = widgets.details_group.first_child() {
            // PreferencesGroup manages rows via add/remove; rebuild clean rows
            if let Some(row) = child.downcast_ref::<adw::ActionRow>() {
                widgets.details_group.remove(row);
            } else {
                break;
            }
        }
        add_detail_row(&widgets.details_group, &gettext("Identifier"), &item.id);
        add_detail_row(&widgets.details_group, &gettext("Source"), &item.source_id);
        add_detail_row(
            &widgets.details_group,
            &gettext("Language Code"),
            &item.language,
        );
        add_detail_row(
            &widgets.details_group,
            &gettext("Short Name"),
            &item.short_name,
        );
        if item.book_count > 0 {
            add_detail_row(
                &widgets.details_group,
                &gettext("Books"),
                &item.book_count.to_string(),
            );
        }
        if item.chapter_count > 0 {
            add_detail_row(
                &widgets.details_group,
                &gettext("Chapters"),
                &item.chapter_count.to_string(),
            );
        }
        if item.verse_count > 0 {
            add_detail_row(
                &widgets.details_group,
                &gettext("Verses"),
                &item.verse_count.to_string(),
            );
        }
    }
}

fn capability_badge(title: &str, icon_name: &str, enabled: bool) -> gtk::Box {
    let pill = gtk::Box::new(gtk::Orientation::Horizontal, 6);
    pill.set_margin_top(6);
    pill.set_margin_bottom(6);
    pill.set_margin_start(12);
    pill.set_margin_end(12);
    pill.add_css_class("card");

    let icon = gtk::Image::from_icon_name(if enabled {
        icon_name
    } else {
        "window-close-symbolic"
    });
    if !enabled {
        icon.add_css_class("dim-label");
    }

    let label = gtk::Label::new(Some(title));
    if !enabled {
        label.add_css_class("dim-label");
    }

    pill.append(&icon);
    pill.append(&label);
    pill
}

fn add_detail_row(group: &adw::PreferencesGroup, title: &str, value: &str) {
    let row = adw::ActionRow::builder().title(title).build();
    let val = gtk::Label::builder()
        .label(value)
        .selectable(true)
        .valign(gtk::Align::Center)
        .build();
    val.add_css_class("dim-label");
    row.add_suffix(&val);
    group.add(&row);
}

fn draw_cover_surface(cr: &gtk::cairo::Context, w: f64, h: f64, item: &StoreItem) {
    let rgb_color = item.safe_rgb();
    let (r, g, b) = (rgb_color.red, rgb_color.green, rgb_color.blue);

    cr.set_source_rgb(r, g, b);
    let _ = cr.rectangle(0.0, 0.0, w, h);
    let _ = cr.fill();

    let grad = gtk::cairo::LinearGradient::new(0.0, 0.0, w, 0.0);
    grad.add_color_stop_rgba(0.0, 0.0, 0.0, 0.0, 0.35);
    grad.add_color_stop_rgba(0.05, 1.0, 1.0, 1.0, 0.15);
    grad.add_color_stop_rgba(0.12, 0.0, 0.0, 0.0, 0.0);
    let _ = cr.set_source(&grad);
    let _ = cr.rectangle(0.0, 0.0, w, h);
    let _ = cr.fill();

    cr.set_source_rgba(1.0, 1.0, 1.0, 0.2);
    let _ = cr.rectangle(0.0, 0.0, 3.0, h);
    let _ = cr.fill();
}
