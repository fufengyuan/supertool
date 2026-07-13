//! LAN event emitter trait — abstracts away Tauri dependency.
//! Allows LanService to work with both Tauri's AppHandle and GPUI's callback.

use serde_json::Value;

/// Trait for emitting events to the UI layer.
/// Implemented for tauri::AppHandle and a simple callback.
pub trait LanEmitter: Send + Sync {
    fn emit(&self, event: &str, payload: Value);
}

/// Simple callback-based emitter for GPUI usage.
pub struct CallbackEmitter<F: Fn(&str, serde_json::Value) + Send + Sync + 'static> {
    callback: F,
}

impl<F: Fn(&str, serde_json::Value) + Send + Sync + 'static> CallbackEmitter<F> {
    pub fn new(callback: F) -> Self {
        Self { callback }
    }
}

impl<F: Fn(&str, Value) + Send + Sync + 'static> LanEmitter for CallbackEmitter<F> {
    fn emit(&self, event: &str, payload: Value) {
        (self.callback)(event, payload);
    }
}

// NoopEmitter in lan_service.rs

// Tauri integration — only compiled when tauri feature is enabled
#[cfg(feature = "tauri-lan")]
impl LanEmitter for tauri::AppHandle {
    fn emit(&self, event: &str, payload: serde_json::Value) {
        use tauri::Emitter;
        let _ = self.emit(event, payload);
    }
}
