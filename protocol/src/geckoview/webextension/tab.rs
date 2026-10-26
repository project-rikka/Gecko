use serde::{Deserialize, Serialize};

use crate::message::WindowId;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CreateTabDetails {
    pub active: Option<bool>,
    pub cookie_store_id: Option<String>,
    pub discarded: Option<bool>,
    pub index: Option<i32>,
    pub open_in_reader_mode: Option<bool>,
    pub pinned: Option<bool>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct UpdateTabDetails {
    pub active: Option<bool>,
    pub auto_discardable: Option<bool>,
    pub highlighted: Option<bool>,
    pub muted: Option<bool>,
    pub pinned: Option<bool>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewTab {
    pub extension_id: String,
    #[serde(default)]
    pub create_properties: CreateTabDetails,
    pub new_session_id: WindowId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTab {
    pub extension_id: String,
    #[serde(default)]
    pub update_properties: UpdateTabDetails,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloseTab {
    pub extension_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenOptionsPage {
    pub extension_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetTabActive {
    pub active: bool,
}

events! {
    NewTab: "GeckoView:WebExtension:NewTab" => ForEmbedder -> bool;
    UpdateTab: "GeckoView:WebExtension:UpdateTab" => ForEmbedder -> ();
    CloseTab: "GeckoView:WebExtension:CloseTab" => ForEmbedder -> ();
    OpenOptionsPage: "GeckoView:WebExtension:OpenOptionsPage" => ForEmbedder -> ();
    SetTabActive: "GeckoView:WebExtension:SetTabActive" => ForGecko;
}
