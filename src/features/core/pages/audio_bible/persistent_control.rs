use adw::prelude::*;
use relm4::prelude::*;
use std::sync::Arc;
use xbible_engine::engines::audio_engine::engine::{AudioEngine, PlaybackState};

// Assuming this lives globally in your application architecture
#[derive(Debug, Clone)]
pub enum AppInputMessage {
    TogglePlayback,
    SkipForward,
    SkipBackward,
    SyncPlaybackState(Option<PlaybackState>),
}

pub struct AudioPersistentControl {
    engine: Option<Arc<AudioEngine>>,
    is_playing: bool,
    current_title: String,
    current_subtitle: String,
    progress_fraction: f64,
}

#[relm4::component(pub)]
impl SimpleComponent for AudioPersistentControl {
    type Init = Option<Arc<AudioEngine>>;
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

            // Rigid layout sizing specs matching the overlay canvas footprint
            set_width_request: 500,
            set_height_request: 68,

            // Pinned premium structural glassmorphic treatment
            inline_css: "
                background: rgba(18, 18, 18, 0.88);
                backdrop-filter: blur(24px);
                border: 1px solid rgba(255, 255, 255, 0.08);
                border-radius: 34px;
                padding: 0 20px;
                box-shadow: 0px 16px 48px rgba(0,0,0,0.6);
            ",

            // 1. LEFT PIECE: Metadata Node Details
            gtk::Box {
                set_orientation: gtk::Orientation::Horizontal,
                set_spacing: 12,
                set_valign: gtk::Align::Center,
                set_hexpand: true,

                gtk::Box {
                    set_width_request: 42,
                    set_height_request: 42,
                    set_overflow: gtk::Overflow::Hidden,
                    inline_css: "border-radius: 10px; background: rgba(255,255,255,0.06);",

                    gtk::Image {
                        set_icon_name: Some("audio-x-generic-symbolic"),
                        set_pixel_size: 22,
                        set_valign: gtk::Align::Center,
                        set_halign: gtk::Align::Center,
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

            // 2. RIGHT PIECE: Tactical Control Operations Deck
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
        engine_context: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            engine: engine_context,
            is_playing: false,
            current_title: "No Selection".to_string(),
            current_subtitle: "Tap a module to play".to_string(),
            progress_fraction: 0.0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppInputMessage::TogglePlayback => {
                self.is_playing = !self.is_playing;
                if let Some(ref audio_engine) = self.engine {
                    // Route directly down into your Core Rust Audio Pipeline
                    audio_engine.toggle_playback();
                }
            }
            AppInputMessage::SkipForward => {
                if let Some(ref audio_engine) = self.engine {
                    audio_engine.skip_forward();
                }
            }
            AppInputMessage::SkipBackward => {
                if let Some(ref audio_engine) = self.engine {
                    audio_engine.skip_backward();
                }
            }
            AppInputMessage::SyncPlaybackState(state) => {
                if let Some(current_state) = state {
                    self.is_playing = current_state.is_playing;
                    self.current_title = current_state.active_text.clone();
                    self.current_subtitle = current_state.active_text;
                    self.progress_fraction = current_state.active_anchor_index as f64; // e.g. 0.0 to 1.0
                }
            }
        }
    }
}
