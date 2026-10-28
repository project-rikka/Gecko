use serde::{Deserialize, Serialize};

flags! {
    pub struct VisitFlags: u32 {
        const TOP_LEVEL = 1 << 0;
        const REDIRECT_TEMPORARY = 1 << 1;
        const REDIRECT_PERMANENT = 1 << 2;
        const REDIRECT_SOURCE = 1 << 3;
        const REDIRECT_SOURCE_PERMANENT = 1 << 4;
        const UNRECOVERABLE_ERROR = 1 << 5;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnVisited {
    pub url: String,
    #[serde(default, rename = "lastVisitedURL")]
    pub last_visited_url: Option<String>,
    pub flags: VisitFlags,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetVisited {
    pub urls: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetHostVisitedSince {
    pub host: String,
    pub after: i64,
    pub before: i64,
}

events! {
    OnVisited: "GeckoView:OnVisited" => ForEmbedder -> bool;
    GetVisited: "GeckoView:GetVisited" => ForEmbedder -> Option<Vec<bool>>;
    GetHostVisitedSince: "GeckoView:GetHostVisitedSince" => ForEmbedder -> Option<bool>;
}
