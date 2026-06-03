//! Minimal stubs replacing claw-code's `runtime` crate.
//! Real implementations exist in the upstream `claw-code` repository;
//! these stubs exist only to let the LLM client code compile as a
//! standalone crate with zero external dependencies beyond reqwest/serde/tokio.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

#[must_use]
pub fn format_usd(amount: f64) -> String {
    format!("${amount:.4}")
}

// ---------------------------------------------------------------------------
// Token usage & cost estimation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cache_creation_input_tokens: u32,
    pub cache_read_input_tokens: u32,
}

impl TokenUsage {
    #[must_use]
    pub const fn total_tokens(&self) -> u32 {
        self.input_tokens
            + self.output_tokens
            + self.cache_creation_input_tokens
            + self.cache_read_input_tokens
    }

    #[must_use]
    pub fn estimate_cost_usd(&self) -> UsageCostEstimate {
        UsageCostEstimate {
            total_cost_usd: 0.0,
        }
    }

    #[must_use]
    pub fn estimate_cost_usd_with_pricing(&self, _pricing: ModelPricing) -> UsageCostEstimate {
        UsageCostEstimate {
            total_cost_usd: 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsageCostEstimate {
    pub total_cost_usd: f64,
}

impl UsageCostEstimate {
    #[must_use]
    pub const fn total_cost_usd(&self) -> f64 {
        self.total_cost_usd
    }
}

// ---------------------------------------------------------------------------
// Pricing
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModelPricing;

#[must_use]
pub fn pricing_for_model(_model: &str) -> Option<ModelPricing> {
    None
}

// ---------------------------------------------------------------------------
// Model family identity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelFamilyIdentity {
    Claude,
    Generic,
}

// ---------------------------------------------------------------------------
// OAuth stubs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub authorize_url: String,
    pub token_url: String,
    pub callback_port: Option<u16>,
    pub manual_redirect_url: Option<String>,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct OAuthTokenSet {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<u64>,
    #[serde(default)]
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct OAuthRefreshRequest;

impl OAuthRefreshRequest {
    #[must_use]
    pub fn from_config(
        _config: &OAuthConfig,
        _refresh_token: String,
        _scopes: Option<Vec<String>>,
    ) -> Self {
        Self
    }

    #[must_use]
    pub fn form_params(&self) -> std::collections::BTreeMap<String, String> {
        std::collections::BTreeMap::new()
    }
}

#[derive(Debug, Clone)]
pub struct OAuthTokenExchangeRequest;

impl OAuthTokenExchangeRequest {
    #[must_use]
    pub fn form_params(&self) -> std::collections::BTreeMap<String, String> {
        std::collections::BTreeMap::new()
    }
}

// ---------------------------------------------------------------------------
// OAuth credential persistence stubs
// ---------------------------------------------------------------------------

pub fn load_oauth_credentials() -> std::io::Result<Option<OAuthTokenSet>> {
    Ok(None)
}

pub fn save_oauth_credentials(_token: &OAuthTokenSet) -> std::io::Result<()> {
    Ok(())
}

pub fn clear_oauth_credentials() -> std::io::Result<()> {
    Ok(())
}

// ---------------------------------------------------------------------------
// ConfigLoader (minimal stub — used by #[cfg(test)] in providers/mod.rs)
// ---------------------------------------------------------------------------

use std::path::PathBuf;

pub struct ConfigLoader {
    _cwd: PathBuf,
    _home: PathBuf,
}

impl ConfigLoader {
    #[must_use]
    pub fn new(cwd: &PathBuf, home: &PathBuf) -> Self {
        Self {
            _cwd: cwd.clone(),
            _home: home.clone(),
        }
    }

    pub fn load(&self) -> Result<EmptyConfig, String> {
        Ok(EmptyConfig)
    }
}

pub struct EmptyConfig;

impl EmptyConfig {
    #[must_use]
    pub fn plugins(&self) -> PluginOverride {
        PluginOverride
    }
}

pub struct PluginOverride;

impl PluginOverride {
    #[must_use]
    pub fn max_output_tokens(&self) -> Option<u32> {
        None
    }
}
