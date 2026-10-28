use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CaptureState {
    #[default]
    Off,
    Live,
    Muted,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CaptureChanged {
    pub camera: CaptureState,
    pub microphone: CaptureState,
    pub screen: CaptureState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StopCapture {
    pub only_screen: bool,
}

events! {
    CaptureChanged: "Rikka:View:Capture" => ForEmbedder;
    StopCapture: "Rikka:View:StopCapture" => ForGecko -> ();
}
