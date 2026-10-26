use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::geckoview::permission::ContentPermission;
use crate::message::WindowId;

flags! {
    pub struct LoadFlags: u32 {
        const BYPASS_CACHE = 1 << 0;
        const BYPASS_PROXY = 1 << 1;
        const EXTERNAL = 1 << 2;
        const ALLOW_POPUPS = 1 << 3;
        const BYPASS_CLASSIFIER = 1 << 4;
        const FORCE_ALLOW_DATA_URI = 1 << 5;
        const REPLACE_HISTORY = 1 << 6;
        const BYPASS_LOAD_URI_DELEGATE = 1 << 7;
    }
}

int_enum! {
    pub enum HeaderFilter: u8 {
        CorsSafelisted = 1,
        UnrestrictedUnsafe = 2,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadUri {
    pub uri: String,
    pub flags: LoadFlags,
    pub header_filter: HeaderFilter,
    pub text_directive_user_activation: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub referrer_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub referrer_session_id: Option<WindowId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<BTreeMap<String, Option<String>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_input: Option<String>,
    pub app_link_launch_type: i32,
}

impl LoadUri {
    pub fn new(uri: impl Into<String>) -> Self {
        Self {
            uri: uri.into(),
            flags: LoadFlags::NONE,
            header_filter: HeaderFilter::CorsSafelisted,
            text_directive_user_activation: false,
            referrer_uri: None,
            referrer_session_id: None,
            headers: None,
            original_input: None,
            app_link_launch_type: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reload {
    pub flags: LoadFlags,
}

empty_payload! {
    pub struct Stop;
    pub struct PurgeHistory;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoBack {
    pub user_interaction: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoForward {
    pub user_interaction: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GotoHistoryIndex {
    pub index: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationChange {
    pub uri: String,
    pub can_go_back: bool,
    pub can_go_forward: bool,
    pub is_top_level: bool,
    #[serde(default)]
    pub permissions: Option<Vec<ContentPermission>>,
    pub has_user_gesture: bool,
}

int_enum! {
    pub enum OpenWhere: u8 {
        DefaultWindow = 0,
        CurrentWindow = 1,
        NewWindow = 2,
        NewTab = 3,
        PrintBrowser = 4,
        NewTabBackground = 5,
        NewTabForeground = 6,
        NewTabAfterCurrent = 7,
    }
}

impl OpenWhere {
    pub const fn opens_new_session(self) -> bool {
        !matches!(self, Self::DefaultWindow | Self::CurrentWindow)
    }
}

flags! {
    pub struct OpenFlags: u32 {
        const EXTERNAL = 0x1;
        const NO_OPENER = 0x4;
        const NO_REFERRER = 0x8;
        const FORCE_ALLOW_DATA_URI = 0x10;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnLoadRequest {
    pub uri: String,
    #[serde(rename = "where")]
    pub target: OpenWhere,
    pub flags: OpenFlags,
    #[serde(default)]
    pub trigger_uri: Option<String>,
    #[serde(default)]
    pub has_user_gesture: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadDecision {
    Allow,
    Deny,
}

impl Serialize for LoadDecision {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bool(*self == Self::Deny)
    }
}

impl<'de> Deserialize<'de> for LoadDecision {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(if bool::deserialize(deserializer)? {
            Self::Deny
        } else {
            Self::Allow
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnNewSession {
    pub uri: String,
    pub new_session_id: WindowId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OnLoadError {
    #[serde(default)]
    pub uri: Option<String>,
    pub error: u32,
    pub error_module: u32,
    pub error_class: u32,
}

impl OnLoadError {
    pub fn classify(&self) -> WebRequestError {
        WebRequestError::from_gecko(self.error, self.error_class)
    }
}

int_enum! {
    pub enum ErrorCategory: u8 {
        Unknown = 0x1,
        Security = 0x2,
        Network = 0x3,
        Content = 0x4,
        Uri = 0x5,
        Proxy = 0x6,
        SafeBrowsing = 0x7,
    }
}

int_enum! {
    pub enum ErrorCode: u8 {
        Unknown = 0x11,
        SecuritySsl = 0x22,
        SecurityBadCert = 0x32,
        NetInterrupt = 0x23,
        NetTimeout = 0x33,
        ConnectionRefused = 0x43,
        UnknownSocketType = 0x53,
        RedirectLoop = 0x63,
        Offline = 0x73,
        PortBlocked = 0x83,
        NetReset = 0x93,
        HttpsOnly = 0xA3,
        BadHstsCert = 0xB3,
        LocalNetworkAccessDenied = 0xC3,
        UnsafeContentType = 0x24,
        CorruptedContent = 0x34,
        ContentCrashed = 0x44,
        InvalidContentEncoding = 0x54,
        UnknownHost = 0x25,
        MalformedUri = 0x35,
        UnknownProtocol = 0x45,
        FileNotFound = 0x55,
        FileAccessDenied = 0x65,
        DataUriTooLong = 0x75,
        ProxyConnectionRefused = 0x26,
        UnknownProxyHost = 0x36,
        SafeBrowsingMalwareUri = 0x27,
        SafeBrowsingUnwantedUri = 0x37,
        SafeBrowsingHarmfulUri = 0x47,
        SafeBrowsingPhishingUri = 0x57,
        HarmfulAddonUri = 0x67,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WebRequestError {
    pub code: ErrorCode,
    pub category: ErrorCategory,
}

const fn failure(module: u32, code: u32) -> u32 {
    (1 << 31) | ((module + 0x45) << 16) | code
}

const MODULE_NETWORK: u32 = 6;
const MODULE_FILES: u32 = 13;
const MODULE_SECURITY: u32 = 21;
const MODULE_URILOADER: u32 = 24;
const MODULE_CONTENT: u32 = 25;

const GECKO_ERRORS: [(u32, ErrorCode); 26] = [
    (
        failure(MODULE_URILOADER, 31),
        ErrorCode::SafeBrowsingPhishingUri,
    ),
    (
        failure(MODULE_URILOADER, 30),
        ErrorCode::SafeBrowsingMalwareUri,
    ),
    (
        failure(MODULE_URILOADER, 35),
        ErrorCode::SafeBrowsingUnwantedUri,
    ),
    (
        failure(MODULE_URILOADER, 38),
        ErrorCode::SafeBrowsingHarmfulUri,
    ),
    (failure(MODULE_URILOADER, 46), ErrorCode::HarmfulAddonUri),
    (failure(MODULE_CONTENT, 16), ErrorCode::ContentCrashed),
    (
        failure(MODULE_NETWORK, 27),
        ErrorCode::InvalidContentEncoding,
    ),
    (failure(MODULE_NETWORK, 74), ErrorCode::UnsafeContentType),
    (failure(MODULE_NETWORK, 29), ErrorCode::CorruptedContent),
    (failure(MODULE_NETWORK, 20), ErrorCode::NetReset),
    (failure(MODULE_NETWORK, 14), ErrorCode::NetTimeout),
    (failure(MODULE_NETWORK, 13), ErrorCode::ConnectionRefused),
    (failure(MODULE_NETWORK, 51), ErrorCode::UnknownSocketType),
    (failure(MODULE_NETWORK, 31), ErrorCode::RedirectLoop),
    (failure(MODULE_NETWORK, 86), ErrorCode::HttpsOnly),
    (failure(MODULE_NETWORK, 89), ErrorCode::BadHstsCert),
    (
        failure(MODULE_NETWORK, 95),
        ErrorCode::LocalNetworkAccessDenied,
    ),
    (failure(MODULE_NETWORK, 16), ErrorCode::Offline),
    (failure(MODULE_NETWORK, 19), ErrorCode::PortBlocked),
    (failure(MODULE_NETWORK, 18), ErrorCode::UnknownProtocol),
    (failure(MODULE_NETWORK, 30), ErrorCode::UnknownHost),
    (failure(MODULE_NETWORK, 10), ErrorCode::MalformedUri),
    (failure(MODULE_FILES, 18), ErrorCode::FileNotFound),
    (failure(MODULE_FILES, 21), ErrorCode::FileAccessDenied),
    (failure(MODULE_NETWORK, 42), ErrorCode::UnknownProxyHost),
    (
        failure(MODULE_NETWORK, 72),
        ErrorCode::ProxyConnectionRefused,
    ),
];

impl WebRequestError {
    pub fn from_gecko(error: u32, error_class: u32) -> Self {
        let in_security_module = (error >> 16) & 0x1fff == MODULE_SECURITY + 0x45;
        let code = GECKO_ERRORS
            .iter()
            .find(|(gecko, _)| *gecko == error)
            .map(|(_, code)| *code)
            .unwrap_or(match (in_security_module, error_class) {
                (true, 1) => ErrorCode::SecuritySsl,
                (true, 2) => ErrorCode::SecurityBadCert,
                _ => ErrorCode::Unknown,
            });
        let category = if in_security_module {
            ErrorCategory::Security
        } else {
            ErrorCategory::from_code(code.code() & 0xF).unwrap_or(ErrorCategory::Unknown)
        };
        Self { code, category }
    }
}

events! {
    LoadUri: "GeckoView:LoadUri" => ForGecko;
    Reload: "GeckoView:Reload" => ForGecko;
    Stop: "GeckoView:Stop" => ForGecko;
    GoBack: "GeckoView:GoBack" => ForGecko;
    GoForward: "GeckoView:GoForward" => ForGecko;
    GotoHistoryIndex: "GeckoView:GotoHistoryIndex" => ForGecko;
    PurgeHistory: "GeckoView:PurgeHistory" => ForGecko;
    LocationChange: "GeckoView:LocationChange" => ForEmbedder;
    OnLoadRequest: "GeckoView:OnLoadRequest" => ForEmbedder -> LoadDecision;
    OnNewSession: "GeckoView:OnNewSession" => ForEmbedder -> bool;
    OnLoadError: "GeckoView:OnLoadError" => ForEmbedder -> String;
}
