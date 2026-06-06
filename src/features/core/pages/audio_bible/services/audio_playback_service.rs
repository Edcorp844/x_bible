pub struct AudioPlaybackService {
    engine: Arc<AudioEngine>,
    hardware_player: Option<HardwareAudioPlayer>,
    pending_player: Arc<Mutex<Option<HardwareAudioPlayer>>>,
    state: SharedPlaybackModel,
    subscribers: Vec<relm4::Sender<SharedPlaybackModel>>,
}
