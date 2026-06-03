use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ===== ROOT LEVEL IMPORTS FOR RODIO 0.22.x =====
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player as RodioPlayer};

use xbible_engine::engines::audio_engine::engine::{AudioEngine, PlaybackState};

// =========================================================================
// HARDWARE AUDIO WRAPPER CONTEXT (Swift AudioBiblePlayer Equivalent)
// =========================================================================
pub struct HardwareAudioPlayer {
    rust_engine: Arc<AudioEngine>,
    player: RodioPlayer,
    _device_sink: MixerDeviceSink, // Keeps hardware audio mixer context alive
    is_interacting: Arc<Mutex<bool>>,
    start_time: Arc<Mutex<Option<(Instant, i64)>>>,
    duration_ms: i64,
}

impl HardwareAudioPlayer {
    pub fn new(
        raw_bytes: Vec<u8>,
        engine: Arc<AudioEngine>,
        duration_ms: i64,
    ) -> Result<Self, String> {
        let device_sink = DeviceSinkBuilder::open_default_sink()
            .map_err(|e| format!("Audio hardware acquisition failed: {:?}", e))?;

        let player = RodioPlayer::connect_new(&device_sink.mixer());
        let cursor = Cursor::new(raw_bytes);
        let decoder = Decoder::try_from(cursor)
            .map_err(|e| format!("Failed to parse decrypted audio container format: {}", e))?;

        player.append(decoder);
        player.pause();

        engine.seek_to_time(0);

        Ok(Self {
            rust_engine: engine,
            player,
            _device_sink: device_sink,
            is_interacting: Arc::new(Mutex::new(false)),
            start_time: Arc::new(Mutex::new(None)),
            duration_ms,
        })
    }

    pub fn play(&self) {
        self.with_interaction_lock(|this| {
            this.player.play();
            let mut start = this.start_time.lock().unwrap();
            if start.is_none() {
                *start = Some((Instant::now(), 0));
            }
            if !this
                .rust_engine
                .get_playback_state()
                .map(|s| s.is_playing)
                .unwrap_or(false)
            {
                this.rust_engine.toggle_playback();
            }
        });
    }

    pub fn pause(&self) {
        self.with_interaction_lock(|this| {
            this.player.pause();
            let mut start = this.start_time.lock().unwrap();
            if let Some((inst, accumulated)) = start.take() {
                *start = Some((
                    Instant::now(),
                    accumulated + inst.elapsed().as_millis() as i64,
                ));
            }
            if this
                .rust_engine
                .get_playback_state()
                .map(|s| s.is_playing)
                .unwrap_or(true)
            {
                this.rust_engine.toggle_playback();
            }
        });
    }

    pub fn stop(&self) {
        self.with_interaction_lock(|this| {
            this.player.pause();
            let _ = this.player.try_seek(Duration::from_millis(0));
            let mut start = this.start_time.lock().unwrap();
            *start = None;
            this.rust_engine.stop();
        });
    }

    pub fn current_time_ms(&self) -> i64 {
        let start = self.start_time.lock().unwrap();
        if let Some((inst, accumulated)) = *start {
            if !self.player.is_paused() {
                return (accumulated + inst.elapsed().as_millis() as i64).min(self.duration_ms);
            }
            return accumulated;
        }
        0
    }

    pub fn seek_to(&self, ms: i64) {
        self.with_interaction_lock(|this| {
            let _ = this.player.try_seek(Duration::from_millis(ms as u64));
            let mut start = this.start_time.lock().unwrap();
            *start = Some((Instant::now(), ms));
            this.rust_engine.seek_to_time(ms);
        });
    }

    pub fn execute_tick_sync(&self) -> Option<PlaybackState> {
        let is_interacting = self.is_interacting.lock().unwrap();
        if *is_interacting {
            return None;
        }

        let current_time = self.current_time_ms();
        let engine_is_playing = self
            .rust_engine
            .get_playback_state()
            .map(|s| s.is_playing)
            .unwrap_or(false);
        let hardware_is_playing = !self.player.is_paused();

        if hardware_is_playing != engine_is_playing {
            self.rust_engine.toggle_playback();
        }

        self.rust_engine.seek_to_time(current_time);
        self.rust_engine.get_playback_state()
    }

    fn with_interaction_lock<F: FnOnce(&Self)>(&self, action: F) {
        if let Ok(mut lock) = self.is_interacting.lock() {
            *lock = true;
        }
        action(self);

        let lock_clone = self.is_interacting.clone();
        glib::timeout_add_local(Duration::from_millis(100), move || {
            if let Ok(mut lock) = lock_clone.lock() {
                *lock = false;
            }
            glib::ControlFlow::Break
        });
    }
}
