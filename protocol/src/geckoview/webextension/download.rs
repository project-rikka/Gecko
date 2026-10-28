use serde::{Deserialize, Serialize};

int_enum! {
    pub enum DownloadState: u8 {
        InProgress = 0,
        Interrupted = 1,
        Complete = 2,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadHeader {
    pub name: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub binary_value: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DownloadOptions {
    pub url: String,
    pub method: Option<String>,
    pub body: Option<String>,
    pub headers: Option<Vec<DownloadHeader>>,
    pub incognito: Option<bool>,
    pub allow_http_errors: Option<bool>,
    pub conflict_action: Option<String>,
    pub save_as: Option<bool>,
    pub filename: Option<String>,
    pub cookie_store_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Download {
    pub options: DownloadOptions,
    pub extension_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadInfo {
    pub id: u32,
    pub bytes_received: i64,
    pub can_resume: bool,
    pub exists: bool,
    pub filename: String,
    pub file_size: i64,
    pub mime: String,
    pub paused: bool,
    #[serde(default)]
    pub referrer: Option<String>,
    pub start_time: String,
    pub state: DownloadState,
    pub total_bytes: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimated_end_time: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadChanged {
    pub download_item_id: u32,
    pub filename: String,
    pub mime: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub state: DownloadState,
    pub can_resume: bool,
    pub paused: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<u8>,
    pub total_bytes: i64,
    pub file_size: i64,
    pub exists: bool,
}

events! {
    Download: "GeckoView:WebExtension:Download" => ForEmbedder -> DownloadInfo;
    DownloadChanged: "GeckoView:WebExtension:DownloadChanged" => ForGecko;
}
