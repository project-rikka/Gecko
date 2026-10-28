use serde::{Deserialize, Serialize};

use crate::geckoview::session::SessionContextId;
use crate::geckoview::storage::ClearFlags;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClearDataBetween {
    pub from: u64,
    pub to: u64,
    pub flags: ClearFlags,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetSiteData {
    pub host: String,
    pub private: bool,
    pub context_id: Option<SessionContextId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiteData {
    pub cookies: u32,
    pub usage: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCookies {
    pub uri: String,
    pub private: bool,
    pub context_id: Option<SessionContextId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SameSite {
    None,
    Lax,
    Strict,
    Unset,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub host: String,
    pub path: String,
    pub secure: bool,
    pub http_only: bool,
    pub expiry: Option<i64>,
    pub same_site: SameSite,
}

events! {
    ClearDataBetween: "Rikka:Storage:ClearBetween" => ForGecko -> ();
    GetSiteData: "Rikka:Storage:SiteData" => ForGecko -> SiteData;
    GetCookies: "Rikka:Storage:Cookies" => ForGecko -> Vec<Cookie>;
}
