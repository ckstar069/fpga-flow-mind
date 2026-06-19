use serde::{Deserialize, Serialize};

use crate::llm::models::{
    DegradedReason, NetworkMode, ProviderCapabilities, ProviderKind, ProviderStatus,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderStatusResponse {
    pub status: ProviderStatus,
    pub kind: ProviderKind,
    pub model: String,
    pub network_mode: NetworkMode,
    pub can_chat: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degraded_reason: Option<DegradedReason>,
    pub capabilities: ProviderCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderValidationResult {
    pub valid: bool,
    pub network_enabled: bool,
    pub issues: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ProviderTestConnectionResult {
    pub success: bool,
    pub code: String,
    pub message: String,
}
