//! supertool-claw — Claw agent LLM client crate
//!
//! Provides only the [`llm`] module — a thin wrapper over the `claw` crate
//! (derived from claw-code's api crate) for use by the Tauri command layer.
//! No subprocess management; all communication is direct HTTP/SSE to the
//! configured LLM provider.

pub mod llm;
