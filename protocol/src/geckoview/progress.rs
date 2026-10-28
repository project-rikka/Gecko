use serde::{Deserialize, Serialize};

int_enum! {
    pub enum IdentityMode: u8 {
        Unknown = 0,
        Identified = 1,
        Verified = 2,
    }
}

int_enum! {
    pub enum MixedMode: u8 {
        Unknown = 0,
        ContentBlocked = 1,
        ContentLoaded = 2,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageStart {
    pub uri: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageStop {
    pub success: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgressChanged {
    pub progress: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityMode {
    pub identity: IdentityMode,
    pub mixed_display: MixedMode,
    pub mixed_active: MixedMode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityIdentity {
    pub mode: SecurityMode,
    #[serde(default)]
    pub origin: Option<String>,
    pub secure: bool,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub certificate: Option<String>,
    #[serde(default)]
    pub security_exception: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityChanged {
    pub identity: SecurityIdentity,
}

empty_payload! {
    pub struct GetQwacStatus;
}

events! {
    PageStart: "GeckoView:PageStart" => ForEmbedder;
    PageStop: "GeckoView:PageStop" => ForEmbedder;
    ProgressChanged: "GeckoView:ProgressChanged" => ForEmbedder;
    SecurityChanged: "GeckoView:SecurityChanged" => ForEmbedder;
    GetQwacStatus: "GeckoView:GetQWACStatus" => ForGecko -> Option<String>;
}
