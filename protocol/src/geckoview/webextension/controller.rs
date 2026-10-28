use serde::{Deserialize, Serialize};

use crate::geckoview::webextension::extension::{Extension, ExtensionReply};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Install {
    pub location_uri: String,
    pub install_id: String,
    #[serde(default)]
    pub install_method: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallFailure {
    pub install_error: i32,
    pub state: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancelled_by_user: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelInstall {
    pub install_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelOutcome {
    pub cancelled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallBuiltIn {
    pub location_uri: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnsureBuiltIn {
    pub location_uri: String,
    pub web_extension_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Uninstall {
    pub web_extension_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnableSource {
    User,
    App,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Enable {
    pub web_extension_id: String,
    pub source: EnableSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Disable {
    pub web_extension_id: String,
    pub source: EnableSource,
}

empty_payload! {
    pub struct List;
    pub struct EnableProcessSpawning;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensionList {
    pub extensions: Vec<Extension>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Get {
    pub extension_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Update {
    pub web_extension_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPrivateBrowsingAllowed {
    pub extension_id: String,
    pub allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddOptionalPermissions {
    pub extension_id: String,
    pub permissions: Vec<String>,
    pub origins: Vec<String>,
    pub data_collection_permissions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveOptionalPermissions {
    pub extension_id: String,
    pub permissions: Vec<String>,
    pub origins: Vec<String>,
    pub data_collection_permissions: Vec<String>,
}

events! {
    Install: "GeckoView:WebExtension:Install" => ForGecko -> ExtensionReply;
    CancelInstall: "GeckoView:WebExtension:CancelInstall" => ForGecko -> CancelOutcome;
    InstallBuiltIn: "GeckoView:WebExtension:InstallBuiltIn" => ForGecko -> ExtensionReply;
    EnsureBuiltIn: "GeckoView:WebExtension:EnsureBuiltIn" => ForGecko -> ExtensionReply;
    Uninstall: "GeckoView:WebExtension:Uninstall" => ForGecko -> ();
    Enable: "GeckoView:WebExtension:Enable" => ForGecko -> ExtensionReply;
    Disable: "GeckoView:WebExtension:Disable" => ForGecko -> ExtensionReply;
    List: "GeckoView:WebExtension:List" => ForGecko -> ExtensionList;
    Get: "GeckoView:WebExtension:Get" => ForGecko -> ExtensionReply;
    Update: "GeckoView:WebExtension:Update" => ForGecko -> Option<ExtensionReply>;
    SetPrivateBrowsingAllowed: "GeckoView:WebExtension:SetPBAllowed" => ForGecko -> ExtensionReply;
    AddOptionalPermissions: "GeckoView:WebExtension:AddOptionalPermissions" => ForGecko -> ExtensionReply;
    RemoveOptionalPermissions: "GeckoView:WebExtension:RemoveOptionalPermissions" => ForGecko -> ExtensionReply;
    EnableProcessSpawning: "GeckoView:WebExtension:EnableProcessSpawning" => ForGecko;
}
