use serde::{Deserialize, Serialize};

use crate::geckoview::session::SessionContextId;

flags! {
    pub struct ClearFlags: u64 {
        const COOKIES = 1 << 0;
        const NETWORK_CACHE = 1 << 1;
        const IMAGE_CACHE = 1 << 2;
        const HISTORY = 1 << 3;
        const DOM_STORAGES = 1 << 4;
        const AUTH_SESSIONS = 1 << 5;
        const PERMISSIONS = 1 << 6;
        const ALL_CACHES = (1 << 1) | (1 << 2);
        const SITE_SETTINGS = (1 << 7) | (1 << 6);
        const SITE_DATA = (1 << 8) | (1 << 0) | (1 << 4) | (1 << 1) | (1 << 2) | (1 << 6) | (1 << 7);
        const ALL = 1 << 9;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClearData {
    pub flags: ClearFlags,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClearHostData {
    pub host: String,
    pub flags: ClearFlags,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearBaseDomainData {
    pub base_domain: String,
    pub flags: ClearFlags,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClearSessionContextData {
    pub context_id: SessionContextId,
}

events! {
    ClearData: "GeckoView:ClearData" => ForGecko -> ();
    ClearHostData: "GeckoView:ClearHostData" => ForGecko -> ();
    ClearBaseDomainData: "GeckoView:ClearBaseDomainData" => ForGecko -> ();
    ClearSessionContextData: "GeckoView:ClearSessionContextData" => ForGecko;
}
