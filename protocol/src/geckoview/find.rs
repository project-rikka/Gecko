use serde::{Deserialize, Serialize};

use crate::geckoview::geometry::Rect;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindInPage {
    pub search_string: Option<String>,
    pub backwards: bool,
    pub links_only: bool,
    pub match_case: bool,
    pub whole_word: bool,
    pub match_diacritics: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct FindFlags {
    pub backwards: bool,
    pub links_only: bool,
    pub match_case: bool,
    pub whole_word: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FindResult {
    pub found: bool,
    pub wrapped: bool,
    pub current: i32,
    pub total: i32,
    pub search_string: String,
    #[serde(default, rename = "linkURL")]
    pub link_url: Option<String>,
    #[serde(default)]
    pub client_rect: Option<Rect>,
    pub flags: FindFlags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayMatches {
    pub highlight_all: bool,
    pub dim_page: bool,
    pub draw_outline: bool,
}

empty_payload! {
    pub struct ClearMatches;
}

events! {
    FindInPage: "GeckoView:FindInPage" => ForGecko -> FindResult;
    DisplayMatches: "GeckoView:DisplayMatches" => ForGecko;
    ClearMatches: "GeckoView:ClearMatches" => ForGecko;
}
