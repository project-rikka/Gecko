use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

flags! {
    pub struct ExtensionFlags: u32 {
        const ALLOW_CONTENT_MESSAGING = 1 << 0;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DisabledFlag {
    UserDisabled,
    BlocklistDisabled,
    SoftBlocklistDisabled,
    AppDisabled,
    SignatureDisabled,
    AppVersionDisabled,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct MetaData {
    #[serde(rename = "amoListingURL")]
    pub amo_listing_url: Option<String>,
    pub average_rating: Option<f64>,
    #[serde(rename = "baseURL")]
    pub base_url: Option<String>,
    pub blocklist_state: Option<u32>,
    pub creator_name: Option<String>,
    #[serde(rename = "creatorURL")]
    pub creator_url: Option<String>,
    pub description: Option<String>,
    pub disabled_flags: Vec<DisabledFlag>,
    pub download_url: Option<String>,
    pub enabled: bool,
    pub full_description: Option<String>,
    #[serde(rename = "homepageURL")]
    pub homepage_url: Option<String>,
    pub icons: Option<BTreeMap<String, String>>,
    pub incognito: Option<String>,
    pub is_recommended: Option<bool>,
    pub name: Option<String>,
    pub open_options_page_in_tab: bool,
    #[serde(rename = "optionsPageURL")]
    pub options_page_url: Option<String>,
    pub private_browsing_allowed: bool,
    pub review_count: Option<u32>,
    #[serde(rename = "reviewURL")]
    pub review_url: Option<String>,
    pub signed_state: Option<i32>,
    pub temporary: bool,
    pub update_date: Option<String>,
    pub version: Option<String>,
    pub required_permissions: Vec<String>,
    pub required_origins: Vec<String>,
    pub required_data_collection_permissions: Vec<String>,
    pub optional_permissions: Vec<String>,
    pub optional_origins: Vec<String>,
    pub optional_data_collection_permissions: Vec<String>,
    pub granted_optional_permissions: Vec<String>,
    pub granted_optional_origins: Vec<String>,
    pub granted_optional_data_collection_permissions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Extension {
    pub web_extension_id: String,
    #[serde(rename = "locationURI")]
    pub location_uri: String,
    pub is_built_in: bool,
    pub web_extension_flags: ExtensionFlags,
    pub meta_data: MetaData,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensionReply {
    pub extension: Extension,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PermissionSet {
    pub permissions: Vec<String>,
    pub origins: Vec<String>,
    pub data_collection: Vec<String>,
}
