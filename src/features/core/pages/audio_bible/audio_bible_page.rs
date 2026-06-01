use std::sync::Arc;
use adw::prelude::*;
use relm4::{Component, ComponentParts, ComponentSender, prelude::*};
use xbible_engine::engines::audio_engine::engine::{AudioEngine, AudioModuleInfo, AudioNode, PlaybackState};


pub struct AudioBiblePage {
    engine: Arc<AudioEngine>,
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
    background_gradient_colors: Vec<(f64, f64, f64, f64)>, // RGBA as f64
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
                gtk::Paned {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_wide_handle: true,
                    set_hexpand: true,
                    set_vexpand: true,

                    #[wrap(Some)]
                    set_start_child = &gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 20,
                        set_margin_all: 24,
                        set_hexpand: false,
                        set_width_request: 300,

                        gtk::Box {
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                            add_css_class: "rounded-container",
                            set_width_request: 250,
                            set_height_request: 250,
                            set_margin_bottom: 20,
                            gtk::Image {
                                set_icon_name: Some("audio-input-microphone-symbolic"),
                                set_pixel_size: 120,
                                set_opacity: 0.3,
                            }
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 8,
                            gtk::Label {
                                set_label: "Audio Module",
                                add_css_class: "title-3",
                                set_wrap: true,
                            },
                            gtk::Label {
                                set_label: "Select a module to play",
                                add_css_class: "subtitle",
                                set_wrap: true,
                            }
                        },

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 6,
                            gtk::Scale {
                                set_draw_value: false,
                                set_hexpand: true,
                                set_range: (0.0, 3600000.0)
                            },
                            gtk::Box {
                                set_orientation: gtk::Orientation::Horizontal,
                                set_spacing: 12,
                                gtk::Label {
                                    set_label: "0:00",
                                    add_css_class: "monospace",
                                    add_css_class: "dim-label",
                                    set_xalign: 0.0,
                                },
                                gtk::Separator {
                                    set_hexpand: true,
                                    set_opacity: 0.0,
                                },
                                gtk::Label {
                                    set_label: "-0:00",
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

                            gtk::Button {
                                set_icon_name: "media-playback-start-symbolic",
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
                        },

                        gtk::ScrolledWindow {
                            set_vexpand: true,
                            set_hscrollbar_policy: gtk::PolicyType::Never,
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
                                set_label: "Chapter Content",
                                add_css_class: "title-3",
                            },

                            gtk::ScrolledWindow {
                                set_vexpand: true,
                                set_hscrollbar_policy: gtk::PolicyType::Never,
                                gtk::Box {
                                    set_orientation: gtk::Orientation::Vertical,
                                    set_spacing: 16,
                                    set_margin_all: 12,
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
            engine: init,
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
            background_gradient_colors: vec![(0.0, 0.0, 0.0, 1.0)],
        };

        let widgets = view_output!();

        let sender_clone = sender.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(30), move || {
            sender_clone.input(AudioBibleInput::UpdatePlaybackState);
            glib::ControlFlow::Continue
        });

        ComponentParts { model, widgets }
    }

    fn update_with_view(
        &mut self,
        _widgets: &mut Self::Widgets,
        message: Self::Input,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            AudioBibleInput::TogglePlayback => {
                if self.is_playing {
                    self.engine.toggle_playback();
                    self.is_playing = false;
                } else {
                    self.engine.toggle_playback();
                    self.is_playing = true;
                }
            }
            AudioBibleInput::Play => {
                self.engine.toggle_playback();
                self.is_playing = true;
            }
            AudioBibleInput::Pause => {
                self.engine.toggle_playback();
                self.is_playing = false;
            }
            AudioBibleInput::Stop => {
                self.engine.stop();
                self.is_playing = false;
                self.current_time_ms = 0;
                self.selected_module = None;
            }
            AudioBibleInput::SkipForward => {
                self.engine.skip_forward();
                self.force_synchronous_state_update();
            }
            AudioBibleInput::SkipBackward => {
                self.engine.skip_backward();
                self.force_synchronous_state_update();
            }
            AudioBibleInput::Seek(time_ms) => {
                self.engine.seek_to_time(time_ms);
                self.current_time_ms = time_ms;
                self.force_synchronous_state_update();
            }
            AudioBibleInput::SelectModule(index) => {
                self.select_module(index);
            }
            AudioBibleInput::UpdatePlaybackState => {
                if let Some(state) = self.engine.get_playback_state() {
                    self.is_playing = state.is_playing;
                    self.current_time_ms = state.current_time_ms;
                    self.active_text = state.active_text.clone();

                    if self.navigation_tree_root.is_none() {
                        self.load_and_cache_navigation_tree();
                    }

                    self.sync_active_chapter(state.current_time_ms);
                }
                self.playback_state = self.engine.get_playback_state();
            }
        }
    }
}

impl AudioBiblePage {
    fn select_module(&mut self, index: usize) {
        self.navigation_tree_root = None;
        self.flattened_chapters_cache.clear();
        self.selected_node_id = None;
        self.is_loading = true;
        self.background_gradient_colors = vec![(0.0, 0.0, 0.0, 1.0)];

        let modules = self.engine.get_audio_modules();
        if let Some(module) = modules.get(index) {
            self.selected_module_index = Some(index);
            self.selected_module = Some(module.clone());

            // Load artwork and extract gradient colors
            self.load_artwork_and_colors(module);

            let base_path = self.engine.get_audio_modules_path();
            let full_path = format!("{}/{}", base_path, &module.file_name);

            if let Ok(_) = self.engine.load_audio_module(full_path.clone()) {
                self.is_loading = false;
                self.engine.toggle_playback();
                self.is_playing = true;
            }
        }
    }

    fn load_artwork_and_colors(&mut self, module: &AudioModuleInfo) {
        // Extract dominant gradient colors from artwork
        let colors = module.artwork.extract_colors(4);
        self.background_gradient_colors = colors
            .iter()
            .map(|c| (c.red, c.green, c.blue, c.alpha))
            .collect();

        // Log artwork info for debugging
        if let Some(artwork_bytes) = module.artwork.image_bytes() {
            println!("🎨 Loaded artwork for module: {} bytes", artwork_bytes.len());
        }
    }

    fn force_synchronous_state_update(&mut self) {
        if let Some(state) = self.engine.get_playback_state() {
            self.is_playing = state.is_playing;
            self.current_time_ms = state.current_time_ms;
            self.active_text = state.active_text.clone();
            self.sync_active_chapter(state.current_time_ms);
            self.playback_state = Some(state);
        }
    }

    fn load_and_cache_navigation_tree(&mut self) {
        if let Some(tree) = self.engine.get_navigation_tree() {
            self.navigation_tree_root = Some(tree.clone());

            self.flattened_chapters_cache = tree.children
                .iter()
                .flat_map(|section| section.children.iter().cloned())
                .collect();

            if self.selected_node_id.is_none() {
                if let Some(first_chapter) = self.flattened_chapters_cache.first() {
                    self.selected_node_id = Some(first_chapter.id.clone());
                }
            }
        }
    }

    fn sync_active_chapter(&mut self, time_ms: i64) {
        if let Some(active_leaf_id) = self.engine.find_active_node_id(time_ms) {
            if let Some(matching_chapter) = self.flattened_chapters_cache.iter()
                .find(|chapter| {
                    chapter.id == active_leaf_id || 
                    chapter.children.iter().any(|child| child.id == active_leaf_id)
                }) 
            {
                if self.selected_node_id != Some(matching_chapter.id.clone()) {
                    self.selected_node_id = Some(matching_chapter.id.clone());
                }
            }
        }
    }

    fn format_time_ms(ms: i64) -> String {
        let total_seconds = (ms / 1000).max(0);
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{}:{:02}", minutes, seconds)
    }

    /// Get the current module display title
    fn get_current_module_title(&self) -> String {
        if let Some(module) = &self.selected_module {
            if let Some(metadata) = &module.metadata {
                return metadata.display_title.clone();
            }
            return module.file_name.clone();
        }
        "Select a module to play".to_string()
    }

    /// Get the current module contributor/artist
    fn get_current_module_contributor(&self) -> String {
        if let Some(module) = &self.selected_module {
            if let Some(metadata) = &module.metadata {
                return metadata.contributor.clone();
            }
        }
        "XBible Media".to_string()
    }

    /// Get all available audio modules
    pub fn get_available_modules(&self) -> Vec<AudioModuleInfo> {
        self.engine.get_audio_modules()
    }

    /// Get the gradient colors as CSS gradient string
    fn get_gradient_css(&self) -> String {
        if self.background_gradient_colors.is_empty() {
            return "linear-gradient(180deg, rgb(0,0,0) 0%, rgb(0,0,0) 100%)".to_string();
        }

        let color_stops: Vec<String> = self.background_gradient_colors
            .iter()
            .enumerate()
            .map(|(idx, (r, g, b, _a))| {
                let percent = if self.background_gradient_colors.len() > 1 {
                    (idx as f64 / (self.background_gradient_colors.len() - 1) as f64) * 100.0
                } else {
                    0.0
                };
                let r_int = (r * 255.0) as u8;
                let g_int = (g * 255.0) as u8;
                let b_int = (b * 255.0) as u8;
                format!("rgb({},{},{}) {:.0}%", r_int, g_int, b_int, percent)
            })
            .collect();

        format!("linear-gradient(135deg, {})", color_stops.join(", "))
    }
}
