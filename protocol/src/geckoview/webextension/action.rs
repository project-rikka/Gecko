use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Action {
    pub title: Option<String>,
    pub icon: Option<BTreeMap<String, String>>,
    pub popup: Option<String>,
    pub badge_text: Option<String>,
    pub badge_background_color: Option<[f64; 4]>,
    pub badge_text_color: Option<[f64; 4]>,
    pub enabled: Option<bool>,
    pub pattern_matching: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionUpdate {
    pub action: Action,
    #[serde(default, rename = "aTabId")]
    pub tab_id: Option<u64>,
    pub extension_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionPopup {
    pub action: Action,
    #[serde(default)]
    pub popup_uri: Option<String>,
    #[serde(default, rename = "aTabId")]
    pub tab_id: Option<u64>,
    pub extension_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BrowserActionUpdate(pub ActionUpdate);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PageActionUpdate(pub ActionUpdate);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct BrowserActionOpenPopup(pub ActionPopup);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PageActionOpenPopup(pub ActionPopup);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserActionClick {
    pub extension_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageActionClick {
    pub extension_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionDelegateAttached {
    pub extension_id: String,
}

events! {
    BrowserActionUpdate: "GeckoView:BrowserAction:Update" => ForEmbedder;
    PageActionUpdate: "GeckoView:PageAction:Update" => ForEmbedder;
    BrowserActionOpenPopup: "GeckoView:BrowserAction:OpenPopup" => ForEmbedder;
    PageActionOpenPopup: "GeckoView:PageAction:OpenPopup" => ForEmbedder;
    BrowserActionClick: "GeckoView:BrowserAction:Click" => ForGecko -> Option<String>;
    PageActionClick: "GeckoView:PageAction:Click" => ForGecko -> Option<String>;
    ActionDelegateAttached: "GeckoView:ActionDelegate:Attached" => ForGecko;
}
