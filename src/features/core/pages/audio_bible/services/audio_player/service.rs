use async_channel::Sender;
use std::sync::{Arc, Mutex};

use relm4::{ComponentSender, Worker};
use xbible_engine::engines::audio_engine::engine::{
    AudioEngine, AudioModuleInfo, AudioNode, PlaybackState,
};

use crate::features::core::pages::audio_bible::{
    audio_bible_page::{AudioBibleInput, ViewControl},
    audio_player::HardwareAudioPlayer,
};

pub struct AudioPlayerService {
    engine: Arc<AudioEngine>,
    hardware_player: Option<HardwareAudioPlayer>,
    pending_player: Arc<Mutex<Option<HardwareAudioPlayer>>>,
    is_playing: bool,
    current_time_ms: i64,
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
    active_text: String,

    subscribers: Vec<Sender<AudioServiceInput>>,
}

#[derive(Debug, Clone)]
pub enum AudioServiceInput {
    // Subscriber Gateway Handshake
    RegisterView(Sender<AudioServiceInput>),

    // Broadcast Notification Payloads
    UpdateSelectedMetadata(AudioModuleInfo),
    SyncPlaybackState(Option<PlaybackState>),
    ModuleLoaded(bool),
    AllModulesExported(Vec<AudioModuleInfo>),

    // Inbound Query Demands
    RequestAllModules,
    RequestCurrentMetadata,

    // Inbound Hardware Actions
    UpdatePlaybackState,
    HandleChapterSeek(String),
    TogglePlayback,
    Stop,
    SkipForward,
    SkipBackward,
    Seek(i64),
    SeekRatio(f64),
    SelectModule(usize),
}

impl AudioPlayerService {
    /// Dispatches state changes out to all connected UI view channels asynchronously
    fn broadcast(&mut self, event: AudioServiceInput) {
        self.subscribers
            .retain(|sub| sub.send_blocking(event.clone()).is_ok());
    }

    pub fn format_time_ms(ms: i64) -> String {
        let total_seconds = 0.max(ms / 1000);
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{}:{:02}", minutes, seconds)
    }

    pub fn get_page_gradient_css(&self) -> String {
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

    pub fn get_current_module_title(&self) -> String {
        if let Some(module) = &self.selected_module {
            if let Some(metadata) = &module.metadata {
                return metadata.display_title.clone();
            }
            return module.file_name.clone();
        }
        "XBible Audio Module".to_string()
    }

    pub fn get_current_module_contributor(&self) -> String {
        if let Some(module) = &self.selected_module {
            if let Some(metadata) = &module.metadata {
                return metadata.contributor.clone();
            }
        }
        "XBible Media".to_string()
    }

    pub fn get_available_modules(&self) -> Vec<AudioModuleInfo> {
        self.available_modules.clone()
    }

    pub fn get_current_module_language(&self) -> String {
        if let Some(module) = &self.selected_module {
            if let Some(metadata) = &module.metadata {
                return metadata.language.clone();
            }
        }
        "Unknown".to_string()
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
            self.broadcast(AudioServiceInput::SyncPlaybackState(Some(state)));
        }
    }
}

impl Worker for AudioPlayerService {
    type Init = Arc<AudioEngine>;
    type Input = AudioServiceInput;
    type Output = AudioServiceInput;

    fn init(engine: Self::Init, sender: ComponentSender<Self>) -> Self {
        let available_modules = engine.get_audio_modules();

        // 🌟 FIX: Instead of calling `glib::timeout_add_local` on a background thread,
        // spawn a standard background thread loop that safely injects updates into the worker channel!
        let worker_input_sender = sender.input_sender().clone();
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(30));
                // Send to the Relm4 worker loop input channel safely.
                // If the worker has shut down, break out of the loop cleanly.
                if worker_input_sender
                    .send(AudioServiceInput::UpdatePlaybackState)
                    .is_err()
                {
                    break;
                }
            }
        });

        AudioPlayerService {
            engine,
            hardware_player: None,
            pending_player: Arc::new(Mutex::new(None)),
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
            available_modules,
            is_stopped: false,
            subscribers: Vec::new(),
        }
    }

    fn update(&mut self, msg: Self::Input, ctx: ComponentSender<Self>) {
        match msg {
            AudioServiceInput::RegisterView(view_sender) => {
                self.subscribers.push(view_sender.clone());

                let modules = self.available_modules.clone();
                let _ = view_sender.send_blocking(AudioServiceInput::AllModulesExported(modules));

                if let Some(ref current_mod) = self.selected_module {
                    let _ = view_sender.send_blocking(AudioServiceInput::UpdateSelectedMetadata(
                        current_mod.clone(),
                    ));
                }
                if self.playback_state.is_some() {
                    let _ = view_sender.send_blocking(AudioServiceInput::SyncPlaybackState(
                        self.playback_state.clone(),
                    ));
                }
            }

            AudioServiceInput::RequestAllModules => {
                let modules = self.available_modules.clone();
                self.broadcast(AudioServiceInput::AllModulesExported(modules));
            }

            AudioServiceInput::RequestCurrentMetadata => {
                if let Some(ref current_mod) = self.selected_module {
                    self.broadcast(AudioServiceInput::UpdateSelectedMetadata(
                        current_mod.clone(),
                    ));
                }
            }

            AudioServiceInput::UpdatePlaybackState => {
                if let Some(ref player) = self.hardware_player {
                    if let Some(state) = player.execute_tick_sync() {
                        self.is_playing = state.is_playing;
                        self.current_time_ms = state.current_time_ms;

                        if self.active_text != state.active_text {
                            self.active_text = state.active_text.clone();
                        }

                        if self.navigation_tree_root.is_none() {
                            self.load_and_cache_navigation_tree();
                        }

                        self.sync_active_chapter(state.current_time_ms);
                        self.playback_state = Some(state.clone());

                        self.broadcast(AudioServiceInput::SyncPlaybackState(Some(state)));
                    }
                }
            }

            AudioServiceInput::HandleChapterSeek(chapter_id) => {
                let target_time = self
                    .flattened_chapters_cache
                    .iter()
                    .find(|c| c.id == chapter_id)
                    .map(|c| c.start_ms.unwrap_or(0));

                self.engine.seek_to_chapter(chapter_id.clone());

                if let Some(timestamp_ms) = target_time {
                    self.current_time_ms = timestamp_ms;
                    if let Some(ref player) = self.hardware_player {
                        player.seek_to(timestamp_ms);
                    }
                } else if let Some(target_state) = self.engine.get_playback_state() {
                    self.current_time_ms = target_state.current_time_ms;
                    if let Some(ref player) = self.hardware_player {
                        player.seek_to(self.current_time_ms);
                    }
                }

                if self.selected_node_id.as_deref() != Some(chapter_id.as_str()) {
                    self.selected_node_id = Some(chapter_id);
                }

                self.force_synchronous_state_update();
            }

            AudioServiceInput::TogglePlayback => {
                if let Some(ref player) = self.hardware_player {
                    if self.is_playing {
                        player.pause();
                        self.is_playing = false;
                    } else {
                        player.play();
                        self.is_playing = true;
                    }
                    self.is_stopped = false;
                    self.force_synchronous_state_update();
                } else {
                    ctx.input_sender()
                        .send(AudioServiceInput::SelectModule(0))
                        .unwrap();
                }
            }

            AudioServiceInput::Stop => {
                if let Some(ref player) = self.hardware_player {
                    player.stop();
                    self.is_playing = false;
                    self.current_time_ms = 0;
                    self.active_text = String::new();
                    self.is_stopped = true;
                    self.force_synchronous_state_update();
                }
            }

            AudioServiceInput::SkipForward => {
                if let Some(ref player) = self.hardware_player {
                    let duration = self
                        .selected_module
                        .as_ref()
                        .and_then(|m| m.metadata.as_ref())
                        .map(|meta| meta.duration_ms)
                        .unwrap_or(3600000);
                    let target_ms = (self.current_time_ms + 30000).min(duration);

                    player.seek_to(target_ms);
                    self.current_time_ms = target_ms;
                    self.force_synchronous_state_update();
                }
            }

            AudioServiceInput::SkipBackward => {
                if let Some(ref player) = self.hardware_player {
                    let target_ms = (self.current_time_ms - 15000).max(0);

                    player.seek_to(target_ms);
                    self.current_time_ms = target_ms;
                    self.force_synchronous_state_update();
                }
            }

            AudioServiceInput::Seek(time_ms) => {
                if let Some(ref player) = self.hardware_player {
                    if (time_ms - self.current_time_ms).abs() > 400 {
                        player.seek_to(time_ms);
                        self.current_time_ms = time_ms;
                        self.force_synchronous_state_update();
                    }
                }
            }

            AudioServiceInput::SeekRatio(ratio) => {
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
                        self.force_synchronous_state_update();
                    }
                }
            }

            AudioServiceInput::SelectModule(index) => {
                self.available_modules = self.engine.get_audio_modules();
                if index < self.available_modules.len() {
                    let module = self.available_modules[index].clone();
                    self.selected_module = Some(module.clone());
                    self.selected_module_index = Some(index);
                    self.is_loading = true;
                    self.navigation_tree_root = None;
                    self.flattened_chapters_cache.clear();
                    self.selected_node_id = None;

                    self.broadcast(AudioServiceInput::UpdateSelectedMetadata(module.clone()));

                    let extracted_colors = module.artwork.extract_colors(4);
                    if !extracted_colors.is_empty() {
                        self.background_gradient_colors = extracted_colors
                            .into_iter()
                            .map(|c| (c.red, c.green, c.blue, c.alpha))
                            .collect();
                    } else {
                        self.background_gradient_colors = vec![(0.1, 0.1, 0.1, 1.0)];
                    }

                    let engine_clone = self.engine.clone();
                    let pending_player_clone = self.pending_player.clone();
                    let worker_sender = ctx.input_sender().clone();
                    let module_path = module.absolute_path.clone();
                    let duration = module
                        .metadata
                        .as_ref()
                        .map(|m| m.duration_ms)
                        .unwrap_or(3600000);

                    std::thread::spawn(move || match engine_clone.load_audio_module(module_path) {
                        Ok(bytes) => {
                            if let Ok(player) =
                                HardwareAudioPlayer::new(bytes, engine_clone, duration)
                            {
                                if let Ok(mut lock) = pending_player_clone.lock() {
                                    *lock = Some(player);
                                }
                                let _ = worker_sender.send(AudioServiceInput::ModuleLoaded(true));
                            } else {
                                let _ = worker_sender.send(AudioServiceInput::ModuleLoaded(false));
                            }
                        }
                        Err(e) => {
                            println!(
                                "[AudioPlayerWorker] ERROR running unpackager layer: {:?}",
                                e
                            );
                            let _ = worker_sender.send(AudioServiceInput::ModuleLoaded(false));
                        }
                    });
                }
            }

            AudioServiceInput::ModuleLoaded(success) => {
                self.is_loading = false;
                if success {
                    let extracted_player = {
                        let mut lock = self.pending_player.lock().unwrap();
                        lock.take()
                    };

                    if let Some(player) = extracted_player {
                        player.play();
                        self.hardware_player = Some(player);
                        self.is_playing = true;
                        self.load_and_cache_navigation_tree();
                        self.force_synchronous_state_update();
                    }
                }

                self.broadcast(AudioServiceInput::ModuleLoaded(success));
            }

            AudioServiceInput::UpdateSelectedMetadata(module_info) => {
                self.broadcast(AudioServiceInput::UpdateSelectedMetadata(module_info));
            }

            AudioServiceInput::SyncPlaybackState(state) => {
                self.broadcast(AudioServiceInput::SyncPlaybackState(state));
            }

            AudioServiceInput::AllModulesExported(modules) => {
                self.broadcast(AudioServiceInput::AllModulesExported(modules));
            }
        }
    }
}
