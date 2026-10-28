use serde::{Deserialize, Serialize};

use crate::geckoview::webextension::extension::Extension;

macro_rules! extension_notices {
    ($($ty:ident: $name:literal;)*) => {
        $(
            #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
            pub struct $ty {
                pub extension: Extension,
            }
        )*

        events! {
            $($ty: $name => ForEmbedder;)*
        }
    };
}

extension_notices! {
    OnInstalling: "GeckoView:WebExtension:OnInstalling";
    OnInstalled: "GeckoView:WebExtension:OnInstalled";
    OnUninstalling: "GeckoView:WebExtension:OnUninstalling";
    OnUninstalled: "GeckoView:WebExtension:OnUninstalled";
    OnEnabling: "GeckoView:WebExtension:OnEnabling";
    OnEnabled: "GeckoView:WebExtension:OnEnabled";
    OnDisabling: "GeckoView:WebExtension:OnDisabling";
    OnDisabled: "GeckoView:WebExtension:OnDisabled";
    OnReady: "GeckoView:WebExtension:OnReady";
    OnOptionalPermissionsChanged: "GeckoView:WebExtension:OnOptionalPermissionsChanged";
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnInstallationFailed {
    #[serde(default)]
    pub extension: Option<Extension>,
    #[serde(default)]
    pub addon_id: Option<String>,
    #[serde(default)]
    pub addon_name: Option<String>,
    #[serde(default)]
    pub addon_version: Option<String>,
    pub error: i32,
}

empty_payload! {
    pub struct OnDisabledProcessSpawning;
    pub struct DebuggerListUpdated;
}

events! {
    OnInstallationFailed: "GeckoView:WebExtension:OnInstallationFailed" => ForEmbedder;
    OnDisabledProcessSpawning: "GeckoView:WebExtension:OnDisabledProcessSpawning" => ForEmbedder;
    DebuggerListUpdated: "GeckoView:WebExtension:DebuggerListUpdated" => ForEmbedder;
}
