use std::sync::{Arc, OnceLock};
use xbible_engine::XBibleEngine;

pub type SharedEngine = Arc<XBibleEngine>;

static ENGINE: OnceLock<SharedEngine> = OnceLock::new();

pub fn init_shared_engine() -> Result<SharedEngine, String> {
    if let Some(existing) = ENGINE.get() {
        return Ok(Arc::clone(existing));
    }

    let created: SharedEngine = XBibleEngine::new();
    let _ = ENGINE.set(Arc::clone(&created));
    Ok(ENGINE.get().cloned().unwrap_or(created))
}

pub fn shared_engine() -> SharedEngine {
    init_shared_engine().expect("XBibleEngine failed to initialize")
}
