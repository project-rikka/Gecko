use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetExtensionIcon {
    pub uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionIcon {
    pub mime_type: Option<String>,
    pub data: String,
}

events! {
    GetExtensionIcon: "Rikka:Extension:Icon" => ForGecko -> ExtensionIcon;
}
