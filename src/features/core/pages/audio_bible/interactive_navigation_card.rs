use gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, prelude::*};
use xbible_engine::engines::audio_engine::engine::AudioNode;

pub struct InteractiveNavigationCard {
    pub current_title: String,
    pub current_subtitle: String,
    pub cached_chapters_list: Vec<AudioNode>,
    pub selected_node_id: Option<String>,
    pub current_playback_ms: i64,
    is_revealed: bool,
    chapters_listbox: Option<gtk::ListBox>,
    row_widgets: Vec<RowWidgetCache>,
}

pub struct RowWidgetCache {
    chapter_id: String,
    row_box: gtk::Box,
    icon: gtk::Image,
    num: gtk::Label,
    title: gtk::Label,
    dur_label: gtk::Label,
    start_ms: i64,
    end_ms: i64,
}

#[derive(Debug, Clone)]
pub enum InteractiveNavigationCardInput {
    ToggleReveal,
    SelectChapter(String),
    UpdatePlaybackTime(i64),
    UpdateChapters(Vec<AudioNode>, String, String),
    SyncSelectedNode(Option<String>),
}

#[relm4::component(pub)]
impl Component for InteractiveNavigationCard {
    type Init = ();
    type Input = InteractiveNavigationCardInput;
    type Output = String; // emits selected chapter id
    type CommandOutput = ();

    view! {
        #[name = "root"]
        gtk::Box {
            #[watch]
            inline_css: &format!(
                "background-color: {}; border-radius: 16px; border: 1px solid rgba(255,255,255,0.1);",
                if model.is_revealed { "rgba(7, 7, 7, 1)" } else { "rgba(7, 7, 7, 0.59)" }
            ),
            set_margin_horizontal: 20,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                inline_css: "background-color: rgba(255, 255, 255, 0.54); border-radius: 16px; padding: 16px; box-shadow: 0px 12px 32px rgba(0,0,0,0.4);",

                // Header
                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_hexpand: true,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 4,
                        set_hexpand: true,
                        set_halign: gtk::Align::Start,

                        gtk::Label {
                            #[watch]
                            set_label: &model.current_title,
                            add_css_class: "title-3",
                            inline_css: "font-size: 14px; font-weight: bold; color: #ffffff;",
                        },
                        gtk::Label {
                            #[watch]
                            set_label: &model.current_subtitle,
                            inline_css: "font-size: 12px; color: rgba(255,255,255,0.5);",
                        },
                    },

                    gtk::Image {
                        #[watch]
                        set_icon_name: Some(if model.is_revealed { "go-down-symbolic" } else { "go-next-symbolic" }),
                        set_valign: gtk::Align::Center,
                    },

                    add_controller = gtk::GestureClick {
                        connect_released[sender] => move |_,_,_,_| {
                            sender.input(InteractiveNavigationCardInput::ToggleReveal);
                        }
                    }
                },

                gtk::Revealer {
                    #[watch]
                    set_reveal_child: model.is_revealed,
                    set_transition_type: gtk::RevealerTransitionType::SlideDown,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_margin_top: 12,
                        set_hexpand: true,

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_hexpand: true,
                            #[watch]
                            set_visible: !model.cached_chapters_list.is_empty(),

                            gtk::ScrolledWindow {
                                set_max_content_height: 260,
                                set_propagate_natural_height: true,

                                #[name = "chapters_listbox"]
                                gtk::ListBox {
                                    set_selection_mode: gtk::SelectionMode::None,
                                    add_css_class: "navigation-sidebar",
                                }
                            }
                        },

                        gtk::Box {
                            set_margin_all: 20,
                            #[watch]
                            set_visible: model.cached_chapters_list.is_empty(),
                            gtk::Label {
                                set_label: "Loading Navigation Catalog...",
                                inline_css: "color: rgba(255,255,255,0.4);",
                            }
                        }
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let mut model = InteractiveNavigationCard {
            is_revealed: false,
            current_title: "Loading...".to_string(),
            current_subtitle: String::new(),
            cached_chapters_list: vec![],
            selected_node_id: None,
            current_playback_ms: 0,
            chapters_listbox: None,
            row_widgets: vec![],
        };

        let widgets = view_output!();

        // Extract a cheap reference clone right out of the generated view structure
        model.chapters_listbox = Some(widgets.chapters_listbox.clone());

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        let mut rebuild = false;
        let mut update = false;

        match message {
            InteractiveNavigationCardInput::ToggleReveal => {
                self.is_revealed = !self.is_revealed;
            }
            InteractiveNavigationCardInput::SelectChapter(id) => {
                self.selected_node_id = Some(id.clone());
                self.is_revealed = false;
                let _ = sender.output(id);
                update = true;
            }
            InteractiveNavigationCardInput::UpdatePlaybackTime(ms) => {
                self.current_playback_ms = ms;
                update = true;
            }
            InteractiveNavigationCardInput::UpdateChapters(chapters, title, subtitle) => {
                let current_first = self.cached_chapters_list.first().map(|c| c.id.clone());
                let new_first = chapters.first().map(|c| c.id.clone());
                if self.cached_chapters_list.len() != chapters.len() || current_first != new_first {
                    rebuild = true;
                }
                self.cached_chapters_list = chapters;
                self.current_title = title;
                self.current_subtitle = subtitle;
            }
            InteractiveNavigationCardInput::SyncSelectedNode(id) => {
                if self.selected_node_id != id {
                    self.selected_node_id = id;
                    update = true;
                }
            }
        }

        if rebuild {
            self.build_chapters_list(&sender);
        } else if update {
            self.update_chapters_list();
        }
    }
}

impl InteractiveNavigationCard {
    fn build_chapters_list(&mut self, sender: &ComponentSender<Self>) {
        let Some(listbox) = &self.chapters_listbox else {
            return;
        };

        while let Some(child) = listbox.first_child() {
            listbox.remove(&child);
        }
        self.row_widgets.clear();

        for (index, chapter) in self.cached_chapters_list.iter().enumerate() {
            let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            row_box.set_margin_start(8);
            row_box.set_margin_end(8);
            row_box.set_margin_top(4);
            row_box.set_margin_bottom(4);

            let icon = gtk::Image::builder()
                .icon_name("audio-volume-high-symbolic")
                .build();

            let num = gtk::Label::new(Some(&(index + 1).to_string()));
            num.inline_css("font-family: monospace; color: rgba(255,255,255,0.5);");

            row_box.append(&icon);
            row_box.append(&num);

            let title = gtk::Label::builder()
                .label(&chapter.title)
                .hexpand(true)
                .halign(gtk::Align::Start)
                .build();
            row_box.append(&title);

            let dur_label = gtk::Label::new(None);
            dur_label.inline_css("font-family: monospace; color: rgba(255,255,255,0.6);");
            row_box.append(&dur_label);

            let row = gtk::ListBoxRow::new();
            row.set_child(Some(&row_box));
            row.set_widget_name(&format!("row_{}", chapter.id));
            row.set_focusable(true);
            row.set_activatable(true);

            let gesture = gtk::GestureClick::new();
            gesture.set_propagation_phase(gtk::PropagationPhase::Capture);

            let sender_clone = sender.clone();
            let chapter_id = chapter.id.clone();

            gesture.connect_released(move |_, _, _, _| {
                sender_clone.input(InteractiveNavigationCardInput::SelectChapter(
                    chapter_id.clone(),
                ));
            });
            row.add_controller(gesture);

            listbox.append(&row);

            self.row_widgets.push(RowWidgetCache {
                chapter_id: chapter.id.clone(),
                row_box,
                icon,
                num,
                title,
                dur_label,
                start_ms: chapter.start_ms.unwrap_or(0),
                end_ms: chapter.end_ms.unwrap_or(0),
            });
        }

        self.update_chapters_list();
    }

    fn update_chapters_list(&self) {
        for widget in &self.row_widgets {
            let is_selected = self.selected_node_id.as_ref() == Some(&widget.chapter_id);

            if is_selected {
                widget.row_box.inline_css("background-color: rgba(255,255,255,0.1); border-radius: 8px; padding: 8px 12px;");
                widget.icon.set_visible(true);
                widget.num.set_visible(false);
                widget.title.inline_css("color: #ffffff; font-weight: 500;");

                let remaining = (widget.end_ms - self.current_playback_ms).max(0);
                let secs = remaining / 1000;
                widget
                    .dur_label
                    .set_label(&format!("-{}:{:02}", secs / 60, secs % 60));
            } else {
                widget.row_box.inline_css(
                    "background-color: transparent; border-radius: 0px; padding: 8px 12px;",
                );
                widget.icon.set_visible(false);
                widget.num.set_visible(true);
                widget.title.inline_css("color: rgba(255,255,255,0.85);");

                let total = ((widget.end_ms - widget.start_ms) / 1000).max(0);
                widget.dur_label.set_label(&format!("{}m", total / 60));
            }
        }
    }
}
