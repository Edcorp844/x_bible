pub struct AudioPlayerService {
    engine: Arc<AudioEngine>,
    hardware_player: Option<HardwareAudioPlayer>, // Hardware layer backplane
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
    view_control: ViewControl,

    active_text: String,
    lyrics_scrollview: Option<gtk::ScrolledWindow>,
    lyrics_box: Option<gtk::Box>,
}
