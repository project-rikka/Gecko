use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

empty_payload! {
    pub struct MediaSessionActivated;
    pub struct MediaSessionDeactivated;
    pub struct PlaybackNone;
    pub struct PlaybackPaused;
    pub struct PlaybackPlaying;
    pub struct Play;
    pub struct Pause;
    pub struct StopMedia;
    pub struct NextTrack;
    pub struct PreviousTrack;
    pub struct SeekForward;
    pub struct SeekBackward;
    pub struct SkipAd;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaImage {
    pub src: String,
    #[serde(default)]
    pub sizes: Option<String>,
    #[serde(default, rename = "type")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct MediaMetadata {
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub artwork: Vec<MediaImage>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataChanged {
    pub metadata: MediaMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionState {
    #[serde(default)]
    pub duration: Option<f64>,
    pub playback_rate: f64,
    pub position: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PositionStateChanged {
    pub state: PositionState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeaturesChanged {
    pub features: BTreeMap<String, bool>,
}

impl FeaturesChanged {
    pub fn supports(&self, key: &str) -> bool {
        self.features.get(key).copied().unwrap_or(false)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioSessionType {
    #[serde(rename = "type")]
    pub session_type: String,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ElementMetadata {
    pub src: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub duration: Option<f64>,
    pub seekable: Option<bool>,
    pub audio_track_count: Option<u32>,
    pub video_track_count: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MediaFullscreen {
    pub metadata: ElementMetadata,
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SeekTo {
    pub time: f64,
    pub fast: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MuteAudio {
    pub mute: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AudioFocusChange {
    Gain,
    SystemTransient,
    SystemPermanent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemAudioFocus {
    pub reason: AudioFocusChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordingDeviceType {
    Camera,
    Microphone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordingStatus {
    Recording,
    Inactive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordingDevice {
    #[serde(rename = "type")]
    pub device_type: RecordingDeviceType,
    pub status: RecordingStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordingStatusChanged {
    pub devices: Vec<RecordingDevice>,
}

events! {
    MediaSessionActivated: "GeckoView:MediaSession:Activated" => ForEmbedder;
    MediaSessionDeactivated: "GeckoView:MediaSession:Deactivated" => ForEmbedder;
    MetadataChanged: "GeckoView:MediaSession:Metadata" => ForEmbedder;
    PositionStateChanged: "GeckoView:MediaSession:PositionState" => ForEmbedder;
    FeaturesChanged: "GeckoView:MediaSession:Features" => ForEmbedder;
    PlaybackNone: "GeckoView:MediaSession:Playback:None" => ForEmbedder;
    PlaybackPaused: "GeckoView:MediaSession:Playback:Paused" => ForEmbedder;
    PlaybackPlaying: "GeckoView:MediaSession:Playback:Playing" => ForEmbedder;
    AudioSessionType: "GeckoView:MediaSession:AudioSessionType" => ForEmbedder;
    MediaFullscreen: "GeckoView:MediaSession:Fullscreen" => ForEmbedder -> bool;
    RecordingStatusChanged: "GeckoView:MediaRecordingStatusChanged" => ForEmbedder;
    Play: "GeckoView:MediaSession:Play" => ForGecko;
    Pause: "GeckoView:MediaSession:Pause" => ForGecko;
    StopMedia: "GeckoView:MediaSession:Stop" => ForGecko;
    NextTrack: "GeckoView:MediaSession:NextTrack" => ForGecko;
    PreviousTrack: "GeckoView:MediaSession:PrevTrack" => ForGecko;
    SeekForward: "GeckoView:MediaSession:SeekForward" => ForGecko;
    SeekBackward: "GeckoView:MediaSession:SeekBackward" => ForGecko;
    SkipAd: "GeckoView:MediaSession:SkipAd" => ForGecko;
    SeekTo: "GeckoView:MediaSession:SeekTo" => ForGecko;
    MuteAudio: "GeckoView:MediaSession:MuteAudio" => ForGecko;
    SystemAudioFocus: "GeckoView:MediaSession:SystemAudioFocus" => ForGecko;
}
