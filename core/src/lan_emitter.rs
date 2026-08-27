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
// 注意：core 不依赖 tauri。如需要在 Tauri 侧实现 LanEmitter，应在 tauri 层实现，而不是在 core 里 cfg。
