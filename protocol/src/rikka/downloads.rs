use std::path::Path;

use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadReason {
    Unhandled,
    Attachment,
    Sniffed,
    Save,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRequest {
    pub url: String,
    pub filename: String,
    pub mime_type: Option<String>,
    pub content_length: Option<u64>,
    pub reason: DownloadReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SaveTo {
    path: String,
}

impl SaveTo {
    pub fn new(path: &Path) -> Option<Self> {
        if !path.is_absolute() {
            return None;
        }
        path.to_str().map(|path| Self {
            path: path.to_owned(),
        })
    }

    pub fn path(&self) -> &Path {
        Path::new(&self.path)
    }
}

impl<'de> Deserialize<'de> for SaveTo {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Wire {
            path: String,
        }
        let Wire { path } = Wire::deserialize(deserializer)?;
        Self::new(Path::new(&path)).ok_or_else(|| {
            serde::de::Error::custom(format_args!("`{path}` is not an absolute path"))
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DownloadId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadState {
    Downloading,
    Paused,
    Finished,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadChanged {
    pub id: DownloadId,
    pub url: String,
    pub referrer: Option<String>,
    pub private: bool,
    pub path: String,
    pub mime_type: Option<String>,
    pub state: DownloadState,
    pub current_bytes: u64,
    pub total_bytes: Option<u64>,
    pub start_time: Option<i64>,
    pub error: Option<String>,
}

macro_rules! download_events {
    ($($ty:ident: $name:literal => $direction:ident $(-> $response:ty)?;)*) => {
        $(
            #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
            pub struct $ty {
                pub id: DownloadId,
            }
        )*

        events! {
            $($ty: $name => $direction $(-> $response)?;)*
        }
    };
}

download_events! {
    DownloadRemoved: "Rikka:Download:Removed" => ForEmbedder;
    CancelDownload: "Rikka:Download:Cancel" => ForGecko -> ();
    PauseDownload: "Rikka:Download:Pause" => ForGecko -> ();
    ResumeDownload: "Rikka:Download:Resume" => ForGecko -> ();
    RemoveDownload: "Rikka:Download:Remove" => ForGecko -> ();
}

events! {
    DownloadRequest: "Rikka:DownloadRequest" => ForEmbedder -> Option<SaveTo>;
    DownloadChanged: "Rikka:Download:Changed" => ForEmbedder;
}
