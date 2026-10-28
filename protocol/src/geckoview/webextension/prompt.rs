use serde::{Deserialize, Serialize};

use crate::geckoview::webextension::extension::{Extension, PermissionSet};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPrompt {
    pub extension: Extension,
    pub permissions: Vec<String>,
    pub origins: Vec<String>,
    pub data_collection_permissions: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallPromptResponse {
    pub allow: bool,
    pub private_browsing_allowed: bool,
    pub is_technical_and_interaction_data_granted: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePrompt {
    pub extension: Extension,
    pub new_permissions: Vec<String>,
    pub new_origins: Vec<String>,
    pub new_data_collection_permissions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionalPrompt {
    pub extension_id: String,
    pub permissions: PermissionSet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllowResponse {
    pub allow: bool,
}

events! {
    InstallPrompt: "GeckoView:WebExtension:InstallPrompt" => ForEmbedder -> InstallPromptResponse;
    UpdatePrompt: "GeckoView:WebExtension:UpdatePrompt" => ForEmbedder -> AllowResponse;
    OptionalPrompt: "GeckoView:WebExtension:OptionalPrompt" => ForEmbedder -> AllowResponse;
}
