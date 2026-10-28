use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetBrowsingDataSettings {
    pub extension_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowsingDataTypes {
    pub cache: bool,
    pub cookies: bool,
    pub downloads: bool,
    pub form_data: bool,
    pub history: bool,
    pub local_storage: bool,
    pub passwords: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemovalOptions {
    pub since: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowsingDataSettings {
    pub options: RemovalOptions,
    pub data_to_remove: BrowsingDataTypes,
    pub data_removal_permitted: BrowsingDataTypes,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BrowsingDataType {
    Downloads,
    FormData,
    History,
    Passwords,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearBrowsingData {
    pub extension_id: String,
    pub data_type: BrowsingDataType,
    #[serde(default)]
    pub since: Option<i64>,
}

events! {
    GetBrowsingDataSettings: "GeckoView:BrowsingData:GetSettings" => ForEmbedder -> BrowsingDataSettings;
    ClearBrowsingData: "GeckoView:BrowsingData:Clear" => ForEmbedder -> ();
}
