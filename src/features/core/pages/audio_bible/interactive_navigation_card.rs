use gtk::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, prelude::*};
use xbible_engine::engines::audio_engine::engine::AudioNode;

pub struct InteractiveNavigationCard {
    pub current_title: String,
    pub current_subtitle: String,
    pub cached_chapters_list: Vec<AudioNode>,
    pub selected_node_id: Option<String>,
    pub current_playback_ms: i64,
    pub live_audio_volume: f32,
    is_revealed: bool,
    // FIX: Keep a direct, cheap reference to the ListBox inside your state struct
    chapters_listbox: Option<gtk::ListBox>,
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
            live_audio_volume: 0.5,
            chapters_listbox: None,
        };

        let widgets = view_output!();

        // Extract a cheap reference clone right out of the generated view structure
        model.chapters_listbox = Some(widgets.chapters_listbox.clone());

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            InteractiveNavigationCardInput::ToggleReveal => {
                self.is_revealed = !self.is_revealed;
            }
            InteractiveNavigationCardInput::SelectChapter(id) => {
                self.selected_node_id = Some(id.clone());
                self.is_revealed = false;
                let _ = sender.output(id);
            }
            InteractiveNavigationCardInput::UpdatePlaybackTime(ms) => {
                self.current_playback_ms = ms;
            }
            InteractiveNavigationCardInput::UpdateChapters(chapters, title, subtitle) => {
                self.cached_chapters_list = chapters;
                self.current_title = title;
                self.current_subtitle = subtitle;
            }
            InteractiveNavigationCardInput::SyncSelectedNode(id) => {
                self.selected_node_id = id;
            }
        }

        self.render_chapters_list(&sender);
    }
}

impl InteractiveNavigationCard {
    fn render_chapters_list(&self, sender: &ComponentSender<Self>) {
        // Safe check to see if the widget handle has been bound yet
        let Some(listbox) = &self.chapters_listbox else {
            return;
        };

        // Clear old rows
        while let Some(child) = listbox.first_child() {
            listbox.remove(&child);
        }

        for (index, chapter) in self.cached_chapters_list.iter().enumerate() {
            let is_selected = self.selected_node_id.as_ref() == Some(&chapter.id);

            let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
            row_box.set_margin_start(8);
            row_box.set_margin_end(8);
            row_box.set_margin_top(4);
            row_box.set_margin_bottom(4);

            if is_selected {
                row_box.inline_css("background-color: rgba(255,255,255,0.1); border-radius: 8px; padding: 8px 12px;");
            } else {
                row_box.inline_css("padding: 8px 12px;");
            }

            // Left side
            if is_selected {
                let icon = gtk::Image::builder()
                    .icon_name("audio-volume-high-symbolic")
                    .build();
                row_box.append(&icon);
            } else {
                let num = gtk::Label::new(Some(&(index + 1).to_string()));
                num.inline_css("font-family: monospace; color: rgba(255,255,255,0.5);");
                row_box.append(&num);
            }

            // Chapter title
            let title = gtk::Label::builder()
                .label(&chapter.title)
                .hexpand(true)
                .halign(gtk::Align::Start)
                .build();
            title.inline_css(if is_selected {
                "color: #ffffff; font-weight: 500;"
            } else {
                "color: rgba(255,255,255,0.85);"
            });
            row_box.append(&title);

            // Duration
            let duration_text = if is_selected {
                let remaining = (chapter.end_ms.unwrap_or(0) - self.current_playback_ms).max(0);
                let secs = remaining / 1000;
                format!("-{}:{:02}", secs / 60, secs % 60)
            } else {
                let total =
                    ((chapter.end_ms.unwrap_or(0) - chapter.start_ms.unwrap_or(0)) / 1000).max(0);
                format!("{}m", total / 60)
            };

            let dur_label = gtk::Label::new(Some(&duration_text));
            dur_label.inline_css("font-family: monospace; color: rgba(255,255,255,0.6);");
            row_box.append(&dur_label);

            let row = gtk::ListBoxRow::new();
            row.set_child(Some(&row_box));
            row.set_widget_name(&format!("row_{}", chapter.id));

            // =========================================================================
            // THE FIX: Add click gesture to make the manual row active
            // =========================================================================
            row.set_focusable(true);
            row.set_activatable(true);

            let gesture = gtk::GestureClick::new();
            let sender_clone = sender.clone();
            let chapter_id = chapter.id.clone();

            gesture.connect_released(move |_, _, _, _| {
                sender_clone.input(InteractiveNavigationCardInput::SelectChapter(
                    chapter_id.clone(),
                ));
            });
            
            // Attach to row_box instead of row, and make sure children don't steal clicks
            row_box.add_controller(gesture);
            if let Some(child) = row_box.first_child() {
                child.set_can_target(false);
                let mut next = child.next_sibling();
                while let Some(n) = next {
                    n.set_can_target(false);
                    next = n.next_sibling();
                }
            }

            listbox.append(&row);
        }
    }
}
