use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::geckoview::session::SessionContextId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PermissionValue {
    Unknown,
    Allow,
    Deny,
    Prompt,
    Other(u32),
}

impl PermissionValue {
    pub const fn code(self) -> u32 {
        match self {
            Self::Unknown => 0,
            Self::Allow => 1,
            Self::Deny => 2,
            Self::Prompt => 3,
            Self::Other(code) => code,
        }
    }

    pub const fn from_code(code: u32) -> Self {
        match code {
            0 => Self::Unknown,
            1 => Self::Allow,
            2 => Self::Deny,
            3 => Self::Prompt,
            other => Self::Other(other),
        }
    }
}

impl Serialize for PermissionValue {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u32(self.code())
    }
}

impl<'de> Deserialize<'de> for PermissionValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        u32::deserialize(deserializer).map(Self::from_code)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PermissionKind {
    Geolocation,
    DesktopNotification,
    PersistentStorage,
    Xr,
    AutoplayInaudible,
    AutoplayAudible,
    MediaKeySystemAccess,
    Tracking,
    StorageAccess,
    LocalDeviceAccess,
    LocalNetworkAccess,
}

impl PermissionKind {
    const STORAGE_PREFIXES: [&'static str; 2] = ["3rdPartyStorage^", "3rdPartyFrameStorage^"];

    pub fn of(perm: &str) -> Option<Self> {
        Some(match perm {
            "geolocation" => Self::Geolocation,
            "desktop-notification" => Self::DesktopNotification,
            "persistent-storage" => Self::PersistentStorage,
            "xr" => Self::Xr,
            "autoplay-media-inaudible" => Self::AutoplayInaudible,
            "autoplay-media-audible" => Self::AutoplayAudible,
            "media-key-system-access" => Self::MediaKeySystemAccess,
            "trackingprotection" | "trackingprotection-pb" => Self::Tracking,
            "loopback-network" => Self::LocalDeviceAccess,
            "local-network" => Self::LocalNetworkAccess,
            "storage-access" => Self::StorageAccess,
            other
                if Self::STORAGE_PREFIXES
                    .iter()
                    .any(|prefix| other.starts_with(prefix)) =>
            {
                Self::StorageAccess
            }
            _ => return None,
        })
    }

    pub const fn perm(self, private_mode: bool) -> &'static str {
        match self {
            Self::Geolocation => "geolocation",
            Self::DesktopNotification => "desktop-notification",
            Self::PersistentStorage => "persistent-storage",
            Self::Xr => "xr",
            Self::AutoplayInaudible => "autoplay-media-inaudible",
            Self::AutoplayAudible => "autoplay-media-audible",
            Self::MediaKeySystemAccess => "media-key-system-access",
            Self::Tracking if private_mode => "trackingprotection-pb",
            Self::Tracking => "trackingprotection",
            Self::StorageAccess => "storage-access",
            Self::LocalDeviceAccess => "loopback-network",
            Self::LocalNetworkAccess => "local-network",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentPermission {
    pub uri: String,
    pub principal: String,
    pub perm: String,
    pub value: PermissionValue,
    #[serde(default)]
    pub context_id: Option<String>,
    pub private_mode: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub third_party_origin: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl ContentPermission {
    pub fn kind(&self) -> Option<PermissionKind> {
        PermissionKind::of(&self.perm)
    }

    pub fn storage_origin(&self) -> Option<&str> {
        PermissionKind::STORAGE_PREFIXES
            .iter()
            .find_map(|prefix| self.perm.strip_prefix(prefix))
            .or(self.third_party_origin.as_deref())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ContentPermissionRequest(pub ContentPermission);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentPermissionShown {
    pub request_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppPermissionRequest {
    pub perms: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSource {
    #[serde(rename = "type")]
    pub device_type: String,
    pub id: String,
    pub raw_id: String,
    pub name: String,
    pub media_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaPermissionRequest {
    pub uri: String,
    #[serde(default)]
    pub video: Option<Vec<MediaSource>>,
    #[serde(default)]
    pub audio: Option<Vec<MediaSource>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MediaPermissionDecision {
    Deny,
    Grant {
        video: Option<String>,
        audio: Option<String>,
    },
}

#[derive(Serialize, Deserialize)]
struct MediaGrant {
    video: Option<String>,
    audio: Option<String>,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum MediaDecisionWire {
    Denied(bool),
    Granted(MediaGrant),
}

impl Serialize for MediaPermissionDecision {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Deny => serializer.serialize_bool(false),
            Self::Grant { video, audio } => MediaGrant {
                video: video.clone(),
                audio: audio.clone(),
            }
            .serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for MediaPermissionDecision {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match MediaDecisionWire::deserialize(deserializer)? {
            MediaDecisionWire::Denied(false) => Ok(Self::Deny),
            MediaDecisionWire::Denied(true) => Err(serde::de::Error::custom(
                "a media permission is granted by naming its devices",
            )),
            MediaDecisionWire::Granted(MediaGrant { video, audio }) => {
                Ok(Self::Grant { video, audio })
            }
        }
    }
}

empty_payload! {
    pub struct GetAllPermissions;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PermissionList {
    pub permissions: Vec<ContentPermission>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPermissionsByUri {
    pub uri: String,
    pub context_id: Option<SessionContextId>,
    pub private_browsing_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPermission {
    #[serde(flatten)]
    pub permission: ContentPermission,
    pub new_value: PermissionValue,
    pub allow_permanent_private_browsing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPermissionByUri {
    pub uri: String,
    pub context_id: Option<SessionContextId>,
    pub private_id: u32,
    pub perm: String,
    pub new_value: PermissionValue,
}

events! {
    ContentPermissionRequest: "GeckoView:ContentPermission" => ForEmbedder -> PermissionValue;
    ContentPermissionShown: "GeckoView:ContentPermissionShown" => ForGecko;
    AppPermissionRequest: "GeckoView:AndroidPermission" => ForEmbedder -> bool;
    MediaPermissionRequest: "GeckoView:MediaPermission" => ForEmbedder -> MediaPermissionDecision;
    GetAllPermissions: "GeckoView:GetAllPermissions" => ForGecko -> PermissionList;
    GetPermissionsByUri: "GeckoView:GetPermissionsByURI" => ForGecko -> PermissionList;
    SetPermission: "GeckoView:SetPermission" => ForGecko;
    SetPermissionByUri: "GeckoView:SetPermissionByURI" => ForGecko;
}
