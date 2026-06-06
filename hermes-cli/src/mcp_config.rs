use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpServerEntry {
    pub name: String,
    pub command: Option<String>,
    pub url: Option<String>,
    #[serde(default)]
    pub supports_parallel_tool_calls: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McpConfig {
    pub servers: Vec<McpServerEntry>,
}
