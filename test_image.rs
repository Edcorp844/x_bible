use xbible_engine::engines::audio_engine::engine::AudioEngine;
use std::sync::Arc;
fn main() {
    let engine = AudioEngine::new();
    let module = engine.load_audio_module("/Users/zoebrooklyn/Library/Application Support/org.flame.xbible/modules/audio/ephesians_2.xba").unwrap();
    let modules = engine.get_available_modules();
    if let Some(m) = modules.first() {
        if let Some(bytes) = m.artwork.image_bytes() {
            println!("Got bytes: {} bytes", bytes.len());
        } else {
            println!("No bytes found in artwork!");
        }
    }
}
