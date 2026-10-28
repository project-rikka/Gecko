use serde::{Deserialize, Serialize};

use crate::message::WindowId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetLocale {
    pub requested_locales: Option<Vec<String>>,
    pub accept_languages: String,
}

empty_payload! {
    pub struct InitialForeground;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceWorkerOpenWindow {
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenedWindow {
    pub is_open: bool,
    pub session_id: WindowId,
}

events! {
    SetLocale: "GeckoView:SetLocale" => ForGecko;
    InitialForeground: "GeckoView:InitialForeground" => ForGecko;
    ServiceWorkerOpenWindow: "GeckoView:ServiceWorkerOpenWindow" => ForEmbedder -> Option<OpenedWindow>;
}
