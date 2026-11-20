use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use xbible_engine::XBibleEngine;

pub type SharedEngine = Arc<XBibleEngine>;

static ENGINE: OnceLock<OnceLock<SharedEngine>> = OnceLock::new();

fn engine_db_path() -> Result<PathBuf, String> {
    let mut dir = glib::user_data_dir();
    dir.push("xbible");

    std::fs::create_dir_all(&dir).map_err(|e| {
        format!(
            "Failed to create XBible data directory {}: {e}",
            dir.display()
        )
    })?;

    dir.push("xbible.db");
    Ok(dir)
}

fn build_engine() -> Result<SharedEngine, String> {
    let db_path = engine_db_path()?;
    let engine: SharedEngine = XBibleEngine::new();

    let path = db_path
        .into_os_string()
        .into_string()
        .map_err(|os_err| format!("Database path is not valid UTF-8: {os_err:?}"))?;

    engine
        .set_db_path(path)
        .map_err(|err| format!("Failed to set database path on XBibleEngine: {err}"))?;

    Ok(engine)
}

pub fn init_shared_engine() -> Result<SharedEngine, String> {
    let slot = ENGINE.get_or_init(OnceLock::new);

    if let Some(existing) = slot.get() {
        return Ok(Arc::clone(existing));
    }

    let created = build_engine()?;
    let _ = slot.set(Arc::clone(&created));

    slot.get()
        .cloned()
        .ok_or_else(|| "Failed to retrieve initialized XBibleEngine".to_string())
}

pub fn try_shared_engine() -> Result<SharedEngine, String> {
    init_shared_engine()
}
