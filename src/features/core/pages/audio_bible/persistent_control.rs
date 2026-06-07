use adw::prelude::*;
use relm4::{WorkerController, prelude::*};
use std::sync::Arc;
use xbible_engine::engines::audio_engine::engine::{AudioEngine, AudioModuleInfo, PlaybackState};

use crate::features::core::pages::audio_bible::services::audio_player::service::{
    AudioPlayerService, AudioServiceInput,
};

#[derive(Debug, Clone)]
pub enum AppInputMessage {
    HandleMetadataBroadcast(AudioModuleInfo),
    SyncPlaybackState(Option<PlaybackState>),
    TogglePlayback,
    SkipForward,
    SkipBackward,
}

pub struct AudioPersistentControl {
    engine: Arc<AudioEngine>,
    is_playing: bool,
    current_title: String,
    current_subtitle: String,
    current_texture: Option<gtk::gdk::Texture>,
    progress_fraction: f64,
    worker: WorkerController<AudioPlayerService>,
}

#[relm4::component(pub)]
impl SimpleComponent for AudioPersistentControl {
    type Init = Arc<AudioEngine>;
    type Input = AppInputMessage;
    type Output = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::End,
            set_margin_bottom: 24,
            set_spacing: 16,

            set_width_request: 500,
            set_height_request: 68,

            inline_css: "
                background: rgba(18, 18, 18, 0.88);
                backdrop-filter: blur(24px);
                border: 1px solid rgba(255, 255, 255, 0.08);
                border-radius: 34px;
                padding: 0 20px;
                box-shadow: 0px 16px 48px rgba(0,0,0,0.6);
            ",

            // 1. LEFT PIECE: Metadata Details + Hi-DPI Scaled Picture Layout
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 12,
                set_valign: gtk::Align::Center,
                set_hexpand: true,

                gtk::Box {
                    set_halign: gtk::Align::Center,
                    set_valign: gtk::Align::Center,
                    add_css_class: "artwork-container",
                    set_overflow: gtk::Overflow::Hidden,
                    set_hexpand: false,
                    set_vexpand: false,
                    set_width_request: 42,
                    set_height_request: 42,
                    inline_css: "background: rgba(255,255,255,0.06); border-radius: 10px;",

                    gtk::Picture {
                        set_halign: gtk::Align::Fill,
                        set_valign: gtk::Align::Fill,
                        set_hexpand: false,
                        set_vexpand: false,

                        set_can_shrink: true,
                        set_content_fit: gtk::ContentFit::Cover,

                        // Constraint bounding box sizes for the UI layout canvas
                        set_width_request: 50,
                        set_height_request: 50,

                        #[watch]
                        set_paintable: model.current_texture.as_ref().map(|t| t.upcast_ref::<gtk::gdk::Paintable>()),
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_valign: gtk::Align::Center,
                    set_spacing: 2,

                    gtk::Label {
                        #[watch]
                        set_label: &model.current_title,
                        inline_css: "font-weight: bold; font-size: 13px; color: #ffffff;",
                        set_halign: gtk::Align::Start,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_max_width_chars: 18,
                    },
                    gtk::Label {
                        #[watch]
                        set_label: &model.current_subtitle,
                        inline_css: "font-size: 11px; color: rgba(255,255,255,0.45);",
                        set_halign: gtk::Align::Start,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_max_width_chars: 22,
                    }
                }
            },

            // 2. RIGHT PIECE: Control Operations Deck
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 6,
                set_valign: gtk::Align::Center,

                gtk::Button {
                    set_icon_name: "media-skip-backward-symbolic",
                    add_css_class: "flat",
                    add_css_class: "circular",
                    set_tooltip_text: Some("Back 15s"),
                    connect_clicked[sender] => move |_| {
                        sender.input(AppInputMessage::SkipBackward);
                    }
                },

                gtk::Button {
                    #[watch]
                    set_icon_name: if model.is_playing { "media-playback-pause-symbolic" } else { "media-playback-start-symbolic" },
                    add_css_class: "circular",
                    inline_css: "
                        button { background: #ffffff; color: #000000; min-width: 42px; min-height: 42px; box-shadow: 0px 4px 12px rgba(0,0,0,0.2); } 
                        button:hover { background: rgba(255,255,255,0.9); }
                    ",
                    connect_clicked[sender] => move |_| {
                        sender.input(AppInputMessage::TogglePlayback);
                    }
                },

                gtk::Button {
                    set_icon_name: "media-skip-forward-symbolic",
                    add_css_class: "flat",
                    add_css_class: "circular",
                    set_tooltip_text: Some("Forward 30s"),
                    connect_clicked[sender] => move |_| {
                        sender.input(AppInputMessage::SkipForward);
                    }
                }
            }
        }
    }

    fn init(
        engine: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let (view_sender, view_receiver) = async_channel::unbounded::<AudioServiceInput>();

        let worker = AudioPlayerService::builder()
            .detach_worker(engine.clone())
            .forward(sender.input_sender(), |_| AppInputMessage::TogglePlayback);

        let _ = worker
            .sender()
            .send(AudioServiceInput::RegisterView(view_sender));

        let input_sender = sender.input_sender().clone();

        // 🌟 FIX: Switched to thread_default context streaming allocation.
        // This stops thread contextual acquisition races between foreground UI loops and your background engine.
        glib::MainContext::ref_thread_default().spawn_local(async move {
            while let Ok(msg) = view_receiver.recv().await {
                match msg {
                    AudioServiceInput::UpdateSelectedMetadata(module_info) => {
                        let _ = input_sender
                            .send(AppInputMessage::HandleMetadataBroadcast(module_info));
                    }
                    AudioServiceInput::SyncPlaybackState(state) => {
                        let _ = input_sender.send(AppInputMessage::SyncPlaybackState(state));
                    }
                    _ => {}
                }
            }
        });

        let model = Self {
            engine,
            is_playing: false,
            current_title: "No Selection".to_string(),
            current_subtitle: "Tap a module to play".to_string(),
            current_texture: None,
            progress_fraction: 0.0,
            worker,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppInputMessage::SyncPlaybackState(state) => {
                if let Some(current_state) = state {
                    self.is_playing = current_state.is_playing;

                    if !current_state.active_text.is_empty() {
                        self.current_subtitle = current_state.active_text;
                    }

                    self.progress_fraction = current_state.active_anchor_index as f64;
                }
            }

            AppInputMessage::HandleMetadataBroadcast(module_info) => {
                if let Some(bytes) = module_info.artwork.image_bytes() {
                    let stream =
                        gtk::gio::MemoryInputStream::from_bytes(&gtk::glib::Bytes::from(&bytes));

                    // 🌟 FIX: Read the stream at a higher, super-sampled scale factor (e.g., 128x128).
                    // This creates a dense pixel buffer map that won't look blurry on retina/high-DPI screens.
                    match gtk::gdk_pixbuf::Pixbuf::from_stream_at_scale(
                        &stream,
                        50,
                        50,
                        true,
                        gtk::gio::Cancellable::NONE,
                    ) {
                        Ok(pixbuf) => {
                            self.current_texture = Some(gtk::gdk::Texture::for_pixbuf(&pixbuf));
                        }
                        Err(e) => {
                            println!(
                                "[AudioPersistentControl] Pixbuf texture conversion error: {}",
                                e
                            );
                            self.current_texture = None;
                        }
                    }
                } else {
                    self.current_texture = None;
                }

                if let Some(ref metadata) = module_info.metadata {
                    println!(
                        "Player Control UI updating metadata layout for: {}",
                        metadata.display_title
                    );
                    self.current_title = metadata.display_title.clone();
                    self.current_subtitle = metadata.contributor.clone();
                } else {
                    self.current_title = module_info.file_name.clone();
                    self.current_subtitle = "Local Module".to_string();
                }
                self.progress_fraction = 0.0;
            }

            AppInputMessage::TogglePlayback => {
                if self.current_title == "No Selection" {
                    let _ = self
                        .worker
                        .sender()
                        .send(AudioServiceInput::SelectModule(0));
                } else {
                    let _ = self.worker.sender().send(AudioServiceInput::TogglePlayback);
                }
            }
            AppInputMessage::SkipForward => {
                let _ = self.worker.sender().send(AudioServiceInput::SkipForward);
            }
            AppInputMessage::SkipBackward => {
                let _ = self.worker.sender().send(AudioServiceInput::SkipBackward);
            }
        }
    }
}
