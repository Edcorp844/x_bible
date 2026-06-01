use adw::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, prelude::*};
use std::sync::Arc;
use xbible_engine::engines::audio_engine::engine::{
    AudioEngine, AudioModuleInfo, AudioNode, PlaybackState,
};

use crate::features::core::pages::audio_bible::audio_player::HardwareAudioPlayer;
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
}

#[derive(Debug, Clone)]
pub enum AudioBibleInput {
    Play,
    Pause,
    Stop,
    SkipForward,
    SkipBackward,
    Seek(i64),
    SelectModule(usize),
    UpdatePlaybackState,
    TogglePlayback,
}

#[relm4::component(pub)]
impl Component for AudioBiblePage {
    type Init = Arc<AudioEngine>;
    type Input = AudioBibleInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        adw::NavigationPage {
            set_title: "Audio Bible",

            #[wrap(Some)]
            set_child = &gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_hexpand: true,
                set_vexpand: true,
                gtk::HeaderBar {},

                #[watch]
                inline_css: &model.get_page_gradient_css(),

                gtk::Paned {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_wide_handle: true,
                    set_hexpand: true,
                    set_vexpand: true,

                    // ===== LEFT PANEL: PLAYER CONTROLS =====
                    #[wrap(Some)]
                    set_start_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 20,
                        set_margin_all: 24,
                        set_hexpand: false,

                        gtk::Box {
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                            add_css_class: "rounded-container",
                            set_width_request: 250,
                            set_height_request: 250,
                            set_overflow: gtk::Overflow::Hidden,
                            set_hexpand: false,
                            set_vexpand: false,
                            set_margin_bottom: 20,
                            inline_css: "background: rgba(255,255,255,0.05); border-radius: 16px; box-shadow: 0px 8px 24px rgba(0,0,0,0.3);",

                            gtk::Picture {
                                set_halign: gtk::Align::Center,
                                set_valign: gtk::Align::Center,
                                set_hexpand: false,
                                set_vexpand: false,
                                set_can_shrink: true,
                                set_content_fit: gtk::ContentFit::Contain,

                                #[watch]
                                set_paintable: model.selected_module.as_ref()
                                    .and_now(|m| m.artwork.image_bytes())
                                    .and_then(|bytes| {
                                        let stream = gtk::gio::MemoryInputStream::from_bytes(&gtk::glib::Bytes::from(&bytes));
                                        match gtk::gdk_pixbuf::Pixbuf::from_stream(&stream, gtk::gio::Cancellable::NONE) {
                                            Ok(pixbuf) => Some(gtk::gdk::Texture::for_pixbuf(&pixbuf)),
                                            Err(e) => {
                                                println!("[AudioBiblePage] ERROR: Failed to decode artwork bytes into Pixbuf: {}", e);
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

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 8,

                            gtk::Label {
                                #[watch]
                                set_label: &model.get_current_module_title(),
                                add_css_class: "title-3",
                                set_wrap: true,
                                set_justify: gtk::Justification::Center,
                            },
                            gtk::Label {
                                #[watch]
                                set_label: &model.get_current_module_contributor(),
                                add_css_class: "subtitle",
                                set_wrap: true,
                                set_justify: gtk::Justification::Center,
                            }
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 6,

                            gtk::Scale {
                                set_draw_value: false,
                                set_hexpand: true,
                                #[watch]
                                set_range: (0.0, model.selected_module.as_ref().and_then(|m| m.metadata.as_ref()).map(|meta| meta.duration_ms as f64).unwrap_or(3600000.0)),
                                #[watch]
                                set_value: model.current_time_ms as f64,

                                connect_value_changed[sender] => move |scale| {
                                    if scale.is_focus() || scale.should_layout() {
                                        sender.input(AudioBibleInput::Seek(scale.value() as i64));
                                    }
                                }
                            },
                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 12,

                                gtk::Label {
                                    #[watch]
                                    set_label: &Self::format_time_ms(model.current_time_ms),
                                    add_css_class: "monospace",
                                    add_css_class: "dim-label",
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
                                    add_css_class: "dim-label",
                                    set_xalign: 1.0,
                                }
                            }
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
                                add_css_class: "suggested-action",
                                set_width_request: 60,
                                set_height_request: 60,
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

                            gtk::Button {
                                set_icon_name: "media-playback-stop-symbolic",
                                set_tooltip_text: Some("Stop"),
                                connect_clicked[sender] => move |_| {
                                    sender.input(AudioBibleInput::Stop);
                                }
                            }
                        },

                        gtk::Separator {},
                        gtk::Label {
                            set_label: "Available Modules",
                            add_css_class: "title-4",
                            set_halign: gtk::Align::Start,
                        },

                        gtk::ScrolledWindow {
                            set_vexpand: true,
                            set_hscrollbar_policy: gtk::PolicyType::Never,

                            #[name = "module_list"]
                            gtk::ListBox {
                                set_selection_mode: gtk::SelectionMode::Single,
                                add_css_class: "navigation-sidebar",
                                connect_row_selected[sender] => move |_listbox, selected_row| {
                                    if let Some(row) = selected_row {
                                        sender.input(AudioBibleInput::SelectModule(row.index() as usize));
                                    }
                                }
                            }
                        }
                    },

                    // ===== RIGHT PANEL: CONTENT TRANSCRIPT STREAM =====
                    #[wrap(Some)]
                    set_end_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_hexpand: true,
                        set_vexpand: true,

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_margin_all: 24,
                            set_spacing: 16,
                            set_hexpand: true,
                            set_vexpand: true,

                            gtk::Label {
                                #[watch]
                                set_label: if model.selected_node_id.is_some() { "Active Chapter Text" } else { "Chapter Content" },
                                add_css_class: "title-3",
                                set_halign: gtk::Align::Start,
                            },

                            gtk::ScrolledWindow {
                                set_vexpand: true,
                                set_hscrollbar_policy: gtk::PolicyType::Never,
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 16,
                                    set_margin_all: 12,

                                    gtk::Label {
                                        #[watch]
                                        set_label: if model.active_text.is_empty() { "No active text streaming available." } else { &model.active_text },
                                        set_wrap: true,
                                        add_css_class: "body",
                                        set_justify: gtk::Justification::Left,
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
        let model = Self {
            engine: init.clone(),
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
            available_modules: init.get_audio_modules(),
        };

        let widgets = view_output!();

        for module in &model.available_modules {
            let row = adw::ActionRow::builder()
                .title(
                    module
                        .metadata
                        .as_ref()
                        .map(|m| m.display_title.clone())
                        .unwrap_or(module.file_name.clone()),
                )
                .subtitle(
                    module
                        .metadata
                        .as_ref()
                        .map(|m| m.contributor.clone())
                        .unwrap_or_else(|| "Audio Module Source".to_string()),
                )
                .build();
            widgets.module_list.append(&row);
        }

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
                }
            }
            AudioBibleInput::Play => {
                if let Some(ref player) = self.hardware_player {
                    player.play();
                    self.is_playing = true;
                }
            }
            AudioBibleInput::Pause => {
                if let Some(ref player) = self.hardware_player {
                    player.pause();
                    self.is_playing = false;
                }
            }
            AudioBibleInput::Stop => {
                if let Some(ref player) = self.hardware_player {
                    player.stop();
                    self.is_playing = false;
                    self.current_time_ms = 0;
                    self.active_text = String::new();
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
