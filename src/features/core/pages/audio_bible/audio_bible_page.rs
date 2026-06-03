use adw::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, prelude::*};
use std::sync::Arc;
use xbible_engine::engines::audio_engine::engine::{
    AudioEngine, AudioModuleInfo, AudioNode, PlaybackState,
};

use crate::features::core::pages::audio_bible::{
    audio_player::HardwareAudioPlayer, interactive_navigation_card::InteractiveNavigationCard,
};
pub struct AudioBiblePage {
    engine: Arc<AudioEngine>,
    hardware_player: Option<HardwareAudioPlayer>, // Hardware layer backplane
    is_playing: bool,
    current_time_ms: i64,
    active_text: String,
    selected_module_index: Option<usize>,
    selected_module: Option<AudioModuleInfo>,
    navigation_tree_root: Option<AudioNode>,
    flattened_chapters_cache: Vec<AudioNode>,
    selected_node_id: Option<String>,
    is_loading: bool,
    playback_state: Option<PlaybackState>,
    background_gradient_colors: Vec<(f64, f64, f64, f64)>,
    available_modules: Vec<AudioModuleInfo>,
    is_stopped: bool,
    view_control: ViewControl,
    interactive_card: Controller<InteractiveNavigationCard>,
    is_sidebar_visible: bool,
}

#[derive(Debug, Clone)]
pub enum AudioBibleInput {
    Stop,
    SkipForward,
    SkipBackward,
    Seek(i64),
    SeekRatio(f64),
    SelectModule(usize),
    UpdatePlaybackState,
    TogglePlayback,
}

#[derive(Debug, Clone)]
pub enum AudioBibleOutput {
    ToggleSidebar,
}

#[derive(Debug, Clone)]
pub enum ViewControl {
    Playlist,
    Lyrics,
}

#[relm4::component(pub)]
impl Component for AudioBiblePage {
    type Init = (Arc<AudioEngine>, bool);
    type Input = AudioBibleInput;
    type Output = AudioBibleOutput;
    type CommandOutput = ();

    view! {
            #[root]
            adw::NavigationPage {
                set_title: "Audio Bible",

                #[watch]
                inline_css: &model.get_page_gradient_css(),

                #[wrap(Some)]
                set_child = &adw::ToolbarView {
                    add_top_bar = &adw::HeaderBar {
                        #[wrap(Some)]
                        set_title_widget = &adw::WindowTitle { set_title: "Audio Bible" },
                        set_show_title: false,
                        add_css_class: "flat",
                        inline_css: "background: transparent; box-shadow: none;",

                        pack_start = &gtk::ToggleButton {
                            set_icon_name: "sidebar-show-symbolic",
                            #[watch]
                            set_active: model.is_sidebar_visible,
                            connect_clicked[sender] => move |_| {
                                let _ = sender.output(AudioBibleOutput::ToggleSidebar);
                            }
                        }
                    },

                    #[wrap(Some)]
                    set_content = &adw::Clamp {
                        set_maximum_size: 1500,
                        set_tightening_threshold: 1000,

                        #[wrap(Some)]
                        set_child = &gtk::Box {
                            set_orientation: gtk::Orientation::Horizontal,
                            set_valign: gtk::Align::Center,
                            set_hexpand: false,
                            set_vexpand: false,
                            inline_css: "background: transparent;",

                            // =========================================================
                            // LEFT PANEL: SIDEBAR PANE
                            // =========================================================
                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 24,
                                set_margin_all: 24,
                                set_vexpand: true,
                                set_hexpand: false,

                                // Rigid proportional width constraint enforced via standard layout methods
                                set_width_request: 340,
                                inline_css: "background: transparent;",

                                #[watch]
                                set_visible: model.is_sidebar_visible,

                                // Rigid Container for Album Art
                                gtk::Box {
                                    set_halign: gtk::Align::Center,
                                    set_valign: gtk::Align::Center,
                                    add_css_class: "artwork-container",
                                    set_overflow: gtk::Overflow::Hidden,
                                    set_hexpand: false,
                                    set_vexpand: false,
                                    set_width_request: 260,
                                    set_height_request: 260,
                                    inline_css: "background: rgba(255,255,255,0.06); border-radius: 20px; box-shadow: 0px 12px 32px rgba(0,0,0,0.4); border: 1px solid rgba(255,255,255,0.1);",

                                    gtk::Picture {
                                        set_halign: gtk::Align::Fill,
                                        set_valign: gtk::Align::Fill,
                                        set_hexpand: false,
                                        set_vexpand: false,

                                        // FORCE THE PICTURE NOT TO EXPAND BEYOND THE COORD LIMITS
                                        set_can_shrink: true,
                                        set_content_fit: gtk::ContentFit::Cover,

                                        set_width_request: 260,
                                        set_height_request: 260,

                                        #[watch]
                                        set_paintable: model.selected_module.as_ref()
                                            .and_now(|m| m.artwork.image_bytes())
                                            .and_then(|bytes| {
                                                let stream = gtk::gio::MemoryInputStream::from_bytes(&gtk::glib::Bytes::from(&bytes));
                                                match gtk::gdk_pixbuf::Pixbuf::from_stream_at_scale(&stream, 260, 260, true, gtk::gio::Cancellable::NONE) {
                                                    Ok(pixbuf) => Some(gtk::gdk::Texture::for_pixbuf(&pixbuf)),
                                                    Err(e) => {
                                                        println!("[AudioBiblePage] ERROR: Failed to decode artwork bytes: {}", e);
                                                        None
                                                    }
                                                }
                                            })
                                            .map(|tex| tex.upcast::<gtk::gdk::Paintable>())
                                            .as_ref(),

                                        #[watch]
                                        set_resource: if model.selected_module.is_none() {
                                            Some("/org/gtk/libgtk/icons/48x48/status/audio-input-microphone-symbolic.symbolic.png")
                                        } else {
                                            None
                                        },
                                    }
                                },

                                // Metadata Block Details
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Horizontal,
                                    set_spacing: 6,
                                    set_halign: gtk::Align::Fill,
                                    set_hexpand: true,

                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,
                                        set_spacing: 6,
                                        set_halign: gtk::Align::Start,
                                        set_hexpand: true,

                                        gtk::Label {
                                            #[watch]
                                            set_label: &model.get_current_module_language(),
                                            set_halign: gtk::Align::Start,
                                            add_css_class: "dimmed-text",
                                            inline_css: "opacity: 0.6; font-size: 0.85rem; font-weight: bold; text-transform: uppercase; tracking: 1px;",
                                            set_wrap: false,
                                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                                        },
                                        gtk::Label {
                                            #[watch]
                                            set_label: &model.get_current_module_title(),
                                            add_css_class: "title-2",
                                            inline_css: "font-weight: 800; font-size: 1.4rem;",
                                            set_halign: gtk::Align::Start,
                                            set_hexpand: false,
                                            set_wrap: false,
                                             set_width_chars: 20,
                                            set_max_width_chars: 30,
                                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                                        },
                                        gtk::Label {
                                            #[watch]
                                            set_label: &model.get_current_module_contributor(),
                                            add_css_class: "subtitle",
                                            inline_css: "opacity: 0.7; font-size: 1rem;",
                                            set_halign: gtk::Align::Start,
                                            set_hexpand: false,
                                            set_wrap: false,
                                            set_width_chars: 20,
                                            set_max_width_chars: 30,
                                            set_ellipsize: gtk::pango::EllipsizeMode::End,
                                        }
                                    },

                                    // Stop Button
                                    gtk::Box {
                                        set_hexpand: false,
                                        set_vexpand: false,
                                        set_valign: gtk::Align::Center,

                                        gtk::Button {
                                            set_icon_name: "media-playback-stop-symbolic",
                                            set_tooltip_text: Some("Stop"),
                                            set_valign: gtk::Align::Center,
                                            set_halign: gtk::Align::End,
                                            connect_clicked[sender] => move |_| {
                                                sender.input(AudioBibleInput::Stop);
                                            }
                                        }
                                    }
                                },

                                // Timeline Control Scrubber Platform Frame
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_hexpand: true,
                                    inline_css: "margin: 10px 0;",

                                    #[name = "timeline_progress"]
                                    gtk::ProgressBar {
                                        set_hexpand: true,
                                        #[watch]
                                        set_fraction: {
                                                let total = model.selected_module.as_ref()
                                                    .and_then(|m| m.metadata.as_ref())
                                                    .map(|meta| meta.duration_ms as f64)
                                                    .unwrap_or(3600000.0);
                                                (model.current_time_ms as f64 / total).clamp(0.0, 1.0)
                                            },

                                        inline_css: "
                                    progressbar trough { background: rgba(255, 255, 255, 0.15); border-radius: 3px; min-height: 4px; }
                                    progressbar progress { background: rgba(255, 255, 255, 1.0); border-radius: 3px; min-height: 4px; }
                                ",

                                        add_controller = gtk::GestureClick {
                                            set_button: 1,

                                            connect_pressed[sender] => move |gesture, _, x, _| {
                                                if let Some(widget) = gesture.widget() {
                                                    let widget_width = widget.width() as f64;
                                                    if widget_width > 0.0 {
                                                        let sanitized_x = x.clamp(0.0, widget_width);
                                                        let click_ratio = sanitized_x / widget_width;
                                                        sender.input(AudioBibleInput::SeekRatio(click_ratio));
                                                    }
                                                }
                                            }
                                        }
                                    }
                                },

                                // Time Label Counters Block
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Horizontal,

                                    gtk::Label {
                                        #[watch]
                                        set_label: &Self::format_time_ms(model.current_time_ms),
                                        add_css_class: "monospace",
                                        inline_css: "opacity: 0.6; font-size: 0.85rem;",
                                        set_xalign: 0.0,
                                    },
                                    gtk::Separator {
                                        set_hexpand: true,
                                        set_opacity: 0.0,
                                    },
                                    gtk::Label {
                                        #[watch]
                                        set_label: &format!("-{}", Self::format_time_ms(
                                                model.selected_module.as_ref().and_then(|m| m.metadata.as_ref()).map(|meta| meta.duration_ms).unwrap_or(0) - model.current_time_ms
                                            )),
                                        add_css_class: "monospace",
                                        inline_css: "opacity: 0.6; font-size: 0.85rem;",
                                        set_xalign: 1.0,
                                    }
                                },

                                // Transport Operational Deck Layout
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Horizontal,
                                    set_spacing: 24,
                                    set_halign: gtk::Align::Center,

                                    gtk::Button {
                                        set_label: "1x",
                                        set_tooltip_text: Some("Change Playback Speed"),
                                        add_css_class: "circular",
                                        add_css_class: "dimmed",
                                        inline_css: "button { background: rgba(255, 255, 255, 0.05); color: rgba(255, 255, 255, 0.7); font-weight: bold; font-size: 0.85rem; padding: 10px; min-width: 44px; min-height: 44px; }"
                                    },

                                    gtk::Box {
                                        set_orientation: gtk::Orientation::Horizontal,
                                        set_spacing: 12,
                                        set_halign: gtk::Align::Center,

                                        gtk::Button {
                                            set_icon_name: "media-skip-backward-symbolic",
                                            set_tooltip_text: Some("Skip backward 15s"),
                                            connect_clicked[sender] => move |_| {
                                                sender.input(AudioBibleInput::SkipBackward);
                                            }
                                        },

                                        #[name = "play_toggle_btn"]
                                        gtk::Button {
                                            #[watch]
                                            set_icon_name: if model.is_playing { "media-playback-pause-symbolic" } else { "media-playback-start-symbolic" },
                                            set_tooltip_text: Some("Play/Pause"),
                                            connect_clicked[sender] => move |_| {
                                                sender.input(AudioBibleInput::TogglePlayback);
                                            }
                                        },

                                        gtk::Button {
                                            set_icon_name: "media-skip-forward-symbolic",
                                            set_tooltip_text: Some("Skip forward 30s"),
                                            connect_clicked[sender] => move |_| {
                                                sender.input(AudioBibleInput::SkipForward);
                                            }
                                        },
                                    },

                                    gtk::Button {
                                        set_icon_name: "media-playlist-repeat-song-symbolic",
                                        add_css_class: "circular",
                                        add_css_class: "dimmed",
                                    }
                                }
                            },

                            // =========================================================
    // RIGHT PANEL: CONTENT ACTION PANE
    // =========================================================
    gtk::Box {
        set_orientation: gtk::Orientation::Vertical,
        set_hexpand: true, // Consumes all leftover space fluidly
        set_vexpand: true,
        inline_css: "background: transparent;",

        // =========================================================
        // OVERLAY WORKSPACE LAYER
        // =========================================================
        gtk::Overlay {
            set_hexpand: true,
            set_vexpand: true,

            // Layer 1: THE MAIN CHILD (Baseline canvas)
            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                set_vexpand: true,
                set_margin_top: 85,

                match model.view_control {
                    // CASE A: NO INSTANTIATED AUDIO SELECTION -> TARGET LISTING SIDEBAR
                    ViewControl::Playlist => gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_hexpand: true,
                        set_vexpand: true,
                        set_margin_all: 24,

                        #[watch]
                        set_visible: model.selected_module.is_none() || model.is_stopped,

                        gtk::Label {
                            set_label: "● ● ○",
                            add_css_class: "",
                            set_halign: gtk::Align::Start,
                            set_margin_bottom: 12,
                            set_margin_horizontal: 24,
                        },

                        gtk::ScrolledWindow {
                            set_vexpand: true,
                            set_hscrollbar_policy: gtk::PolicyType::Never,

                            #[name = "module_list"]
                            gtk::ListBox {
                                set_selection_mode: gtk::SelectionMode::Single,
                                add_css_class: "navigation-sidebar",
                                inline_css: "background: transparent;",
                                connect_row_selected[sender] => move |_listbox, selected_row| {
                                    if let Some(row) = selected_row {
                                        sender.input(AudioBibleInput::SelectModule(row.index() as usize));
                                    }
                                }
                            }
                        }
                    },

                    // CASE B: MODULE FOCUS ACTIVE -> COMPUTE DYNAMIC LYRICS / TEXT STRINGS
                    ViewControl::Lyrics => gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_hexpand: true,
                        set_vexpand: true,
                        set_spacing: 16,
                        set_margin_all: 24,

                        #[watch]
                        set_visible: model.selected_module.is_some(),

                        gtk::Label {
                            #[watch]
                            set_label: if model.selected_node_id.is_some() { "Active Chapter Text" } else { "Chapter Content" },
                            add_css_class: "title-3",
                            inline_css: "font-weight: bold; opacity: 0.9;",
                            set_halign: gtk::Align::Start,
                        },

                        gtk::ScrolledWindow {
                            set_vexpand: true,
                            set_hscrollbar_policy: gtk::PolicyType::Never,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 16,
                                set_margin_all: 8,

                                gtk::Label {
                                    #[watch]
                                    set_label: if model.active_text.is_empty() { "No active text streaming available." } else { &model.active_text },
                                    set_wrap: true,
                                    add_css_class: "body",
                                    inline_css: "font-size: 1.25rem; line-height: 1.75; opacity: 0.95;",
                                    set_justify: gtk::Justification::Left,
                                }
                            }
                        }
                    },
                }
            },

            // Layer 2: THE FLOATING OVERLAY CONTAINER
            // This configuration perfectly replicates the study-page design mechanics!
            add_overlay = &gtk::Box {
                set_halign: gtk::Align::Fill,  // Stretch horizontally to fit the column width
                set_valign: gtk::Align::Start, // Pin tightly to the top edge
                set_vexpand: false,            // CRITICAL: Tells GTK not to hog vertical space
                set_margin_all: 24,

                #[name = "interactive_navigation_card"]
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_halign: gtk::Align::Fill,
                    inline_css: "background: transparent;",
                }
            }
        }
    }
                        }
                    }
                }
            }
        }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (engine, is_sidebar_visible) = init;
        let interactive_card =
            InteractiveNavigationCard::builder()
                .launch(())
                .forward(sender.input_sender(), |_| {
                    // Map child outputs to parent inputs if needed, or leave empty
                    unreachable!()
                });
        let model = Self {
            engine: engine.clone(),
            hardware_player: None,
            is_playing: false,
            current_time_ms: 0,
            active_text: String::new(),
            selected_module_index: None,
            selected_module: None,
            navigation_tree_root: None,
            flattened_chapters_cache: Vec::new(),
            selected_node_id: None,
            is_loading: false,
            playback_state: None,
            background_gradient_colors: vec![(0.1, 0.1, 0.1, 1.0)],
            available_modules: engine.get_audio_modules(),
            view_control: ViewControl::Playlist,
            is_stopped: false,
            interactive_card,
            is_sidebar_visible,
        };

        let widgets = view_output!();

        for module in &model.available_modules {
            // 1. CONSTRUCT THE START ARTWORK (PREFIX)
            let artwork_frame = gtk::Box::builder()
                .width_request(54)
                .height_request(54)
                .valign(gtk::Align::Center)
                .halign(gtk::Align::Center)
                .margin_top(2)
                .margin_bottom(2)
                .margin_end(2)
                .overflow(gtk::Overflow::Hidden)
                .build();

            artwork_frame
                .inline_css("border-radius: 10px; background-color: rgba(255,255,255,0.05);");

            let artwork_image = gtk::Image::builder()
                .pixel_size(54)
                .valign(gtk::Align::Center)
                .halign(gtk::Align::Center)
                .build();

            // Try to unpack the raw image data from your secure wrapper
            if let Some(bytes) = module.artwork.image_bytes() {
                let stream =
                    gtk::gio::MemoryInputStream::from_bytes(&gtk::glib::Bytes::from(&bytes));
                match gtk::gdk_pixbuf::Pixbuf::from_stream_at_scale(
                    &stream,
                    54,
                    54,
                    false,
                    gtk::gio::Cancellable::NONE,
                ) {
                    Ok(pixbuf) => {
                        let texture = gtk::gdk::Texture::for_pixbuf(&pixbuf);
                        artwork_image.set_paintable(Some(&texture));
                    }
                    Err(e) => {
                        println!("[AudioBiblePage] Fallback icon used. Pixbuf error: {}", e);
                        artwork_image.set_icon_name(Some("audio-x-generic-symbolic"));
                    }
                }
            } else {
                // Safe placeholder icon if no media asset exists
                artwork_image.set_icon_name(Some("audio-x-generic-symbolic"));
            }

            artwork_frame.append(&artwork_image);

            // 2. CONSTRUCT THE ELLIPSIS BUTTON (SUFFIX)
            let ellipsis_button = gtk::MenuButton::builder()
                .icon_name("view-more-horizontal-symbolic")
                .valign(gtk::Align::Center)
                .halign(gtk::Align::End)
                .has_frame(false) // Gives it a clean, flat look until hovered
                .tooltip_text("Module Options")
                .build();

            // 3. BUILD AND ASSEMBLE THE ACTION ROW
            let row = adw::ActionRow::builder()
                .title(
                    module
                        .metadata
                        .as_ref()
                        .map(|m| m.display_title.clone())
                        .unwrap_or_else(|| module.file_name.clone()),
                )
                .subtitle(format!(
                    "{} • {}",
                    module
                        .metadata
                        .as_ref()
                        .map(|m| m.contributor.clone())
                        .unwrap_or_else(|| "Audio Module Source".to_string()),
                    module
                        .metadata
                        .as_ref()
                        .map(|m| m.language.clone())
                        .unwrap_or_else(|| "English".to_string())
                ))
                .margin_bottom(2)
                .build();

            // Attach components to their respective sides
            row.add_prefix(&artwork_frame);
            row.add_suffix(&ellipsis_button);

            // Append the fully decorated row into your list box
            widgets.module_list.append(&row);
        }

        widgets
            .interactive_navigation_card
            .append(model.interactive_card.widget());

        // Establish core hardware monitoring heartbeat loops every 30ms
        let sender_clone = sender.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(30), move || {
            sender_clone.input(AudioBibleInput::UpdatePlaybackState);
            glib::ControlFlow::Continue
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        match message {
            AudioBibleInput::TogglePlayback => {
                if let Some(ref player) = self.hardware_player {
                    if self.is_playing {
                        player.pause();
                        self.is_playing = false;
                    } else {
                        player.play();
                        self.is_playing = true;
                    }
                    self.is_stopped = false;
                }
            }

            AudioBibleInput::Stop => {
                if let Some(ref player) = self.hardware_player {
                    player.stop();
                    self.is_playing = false;
                    self.current_time_ms = 0;
                    self.active_text = String::new();
                    self.is_stopped = true;
                }
            }
            AudioBibleInput::SkipForward => {
                if let Some(ref player) = self.hardware_player {
                    let target_ms = (self.current_time_ms + 30000).min(
                        self.selected_module
                            .as_ref()
                            .and_then(|m| m.metadata.as_ref())
                            .map(|meta| meta.duration_ms)
                            .unwrap_or(3600000),
                    );
                    player.seek_to(target_ms);
                    self.current_time_ms = target_ms;
                    self.force_synchronous_state_update();
                }
            }
            AudioBibleInput::SkipBackward => {
                if let Some(ref player) = self.hardware_player {
                    let target_ms = (self.current_time_ms - 15000).max(0);
                    player.seek_to(target_ms);
                    self.current_time_ms = target_ms;
                    self.force_synchronous_state_update();
                }
            }
            AudioBibleInput::Seek(time_ms) => {
                if let Some(ref player) = self.hardware_player {
                    if (time_ms - self.current_time_ms).abs() > 400 {
                        player.seek_to(time_ms);
                        self.current_time_ms = time_ms;
                    }
                }
            }
            AudioBibleInput::SeekRatio(ratio) => {
                if let Some(ref player) = self.hardware_player {
                    let duration = self
                        .selected_module
                        .as_ref()
                        .and_then(|m| m.metadata.as_ref())
                        .map(|meta| meta.duration_ms)
                        .unwrap_or(0);
                    let target_ms = (ratio * duration as f64) as i64;
                    if (target_ms - self.current_time_ms).abs() > 400 {
                        player.seek_to(target_ms);
                        self.current_time_ms = target_ms;
                    }
                }
            }
            AudioBibleInput::SelectModule(index) => {
                self.select_module(index);
            }
            AudioBibleInput::UpdatePlaybackState => {
                if let Some(ref player) = self.hardware_player {
                    if let Some(state) = player.execute_tick_sync() {
                        self.is_playing = state.is_playing;
                        self.current_time_ms = state.current_time_ms;
                        self.active_text = state.active_text.clone();

                        if self.navigation_tree_root.is_none() {
                            self.load_and_cache_navigation_tree();
                        }
                        self.sync_active_chapter(state.current_time_ms);
                        self.playback_state = Some(state);
                    }
                }
            }
        }
    }
}

// Option extension utility block
trait OptionExt<T> {
    fn and_now<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&T) -> Option<R>;
}
impl<T> OptionExt<T> for Option<T> {
    fn and_now<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&T) -> Option<R>,
    {
        self.as_ref().and_then(f)
    }
}

impl AudioBiblePage {
    fn format_time_ms(ms: i64) -> String {
        let total_seconds = 0.max(ms / 1000);
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{}:{:02}", minutes, seconds)
    }

    fn get_page_gradient_css(&self) -> String {
        if self.background_gradient_colors.is_empty() {
            return "background: rgba(0,0,0,1);".to_string();
        }
        let mut stops = Vec::new();
        let len = self.background_gradient_colors.len();

        if len == 1 {
            let c = self.background_gradient_colors[0];
            let rgba_str = format!(
                "rgba({}, {}, {}, {})",
                (c.0 * 255.0) as u8,
                (c.1 * 255.0) as u8,
                (c.2 * 255.0) as u8,
                c.3
            );
            stops.push(format!("{} 0%", rgba_str));
            stops.push(format!("{} 100%", rgba_str));
        } else {
            for (i, c) in self.background_gradient_colors.iter().rev().enumerate() {
                let percentage = (i as f64 / (len - 1) as f64) * 100.0;
                stops.push(format!(
                    "rgba({}, {}, {}, {}) {:.1}%",
                    (c.0 * 255.0) as u8,
                    (c.1 * 255.0) as u8,
                    (c.2 * 255.0) as u8,
                    c.3,
                    percentage
                ));
            }
        }
        let linear_grad = format!("linear-gradient(135deg, {})", stops.join(", "));
        format!(
            "background-image: linear-gradient(rgba(0,0,0,0.45), rgba(0,0,0,0.45)), {};",
            linear_grad
        )
    }

    fn get_current_module_title(&self) -> String {
        if let Some(module) = &self.selected_module {
            if let Some(metadata) = &module.metadata {
                return metadata.display_title.clone();
            }
            return module.file_name.clone();
        }
        "XBible Audio Module".to_string()
    }

    fn get_current_module_contributor(&self) -> String {
        if let Some(module) = &self.selected_module {
            if let Some(metadata) = &module.metadata {
                return metadata.contributor.clone();
            }
        }
        "XBible Media".to_string()
    }

    fn get_available_modules(&self) -> Vec<AudioModuleInfo> {
        self.available_modules.clone()
    }

    fn get_current_module_language(&self) -> String {
        if let Some(module) = &self.selected_module {
            if let Some(metadata) = &module.metadata {
                return metadata.language.clone();
            }
        }
        "Unknown".to_string()
    }

    fn select_module(&mut self, index: usize) {
        let modules = self.get_available_modules();
        if index < modules.len() {
            let module = modules[index].clone();
            self.selected_module = Some(module.clone());
            self.selected_module_index = Some(index);
            self.is_loading = true;
            self.navigation_tree_root = None;
            self.flattened_chapters_cache.clear();
            self.selected_node_id = None;

            let extracted_colors = module.artwork.extract_colors(4);
            if !extracted_colors.is_empty() {
                self.background_gradient_colors = extracted_colors
                    .into_iter()
                    .map(|c| (c.red, c.green, c.blue, c.alpha))
                    .collect();
            } else {
                self.background_gradient_colors = vec![(0.1, 0.1, 0.1, 1.0)];
            }

            // Route decoded streams back into hardware audio architecture
            match self.engine.load_audio_module(module.absolute_path.clone()) {
                Ok(bytes) => {
                    let duration = module
                        .metadata
                        .as_ref()
                        .map(|m| m.duration_ms)
                        .unwrap_or(3600000);
                    if let Ok(player) =
                        HardwareAudioPlayer::new(bytes, self.engine.clone(), duration)
                    {
                        player.play();
                        self.hardware_player = Some(player);
                        self.is_playing = true;
                        self.load_and_cache_navigation_tree();
                        self.force_synchronous_state_update();
                    }
                }
                Err(e) => {
                    println!("[AudioBiblePage] ERROR loading audio module: {:?}", e);
                }
            }
            self.is_loading = false;
        }
    }

    fn load_and_cache_navigation_tree(&mut self) {
        if let Some(tree) = self.engine.get_navigation_tree() {
            self.navigation_tree_root = Some(tree.clone());
            self.flattened_chapters_cache = tree
                .children
                .iter()
                .flat_map(|c| c.children.clone())
                .collect();

            if self.selected_node_id.is_none() {
                if let Some(first) = self.flattened_chapters_cache.first() {
                    self.selected_node_id = Some(first.id.clone());
                }
            }
        }
    }

    fn sync_active_chapter(&mut self, time_ms: i64) {
        if let Some(active_id) = self.engine.find_active_node_id(time_ms) {
            if let Some(matching) = self
                .flattened_chapters_cache
                .iter()
                .find(|c| c.id == active_id || c.children.iter().any(|child| child.id == active_id))
            {
                if self.selected_node_id.as_deref() != Some(&matching.id) {
                    self.selected_node_id = Some(matching.id.clone());
                }
            }
        }
    }

    fn force_synchronous_state_update(&mut self) {
        if let Some(state) = self.engine.get_playback_state() {
            self.playback_state = Some(state.clone());
            self.sync_active_chapter(state.current_time_ms);
        }
    }
}
