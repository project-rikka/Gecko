use serde::{Deserialize, Serialize};

use crate::geckoview::navigation::OpenWhere;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenLink {
    pub uri: String,
    pub referrer: Option<String>,
    #[serde(rename = "where")]
    pub target: OpenWhere,
}

events! {
    OpenLink: "Rikka:View:OpenLink" => ForEmbedder;
}
