use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SenderEnvironment {
    AddonChild,
    ContentChild,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageSender {
    pub id: String,
    pub env_type: SenderEnvironment,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub frame_id: Option<i64>,
}

impl MessageSender {
    pub fn is_top_level(&self) -> bool {
        self.env_type == SenderEnvironment::AddonChild || self.frame_id == Some(0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeMessage {
    pub sender: MessageSender,
    #[serde(default)]
    pub data: Value,
    pub extension_id: String,
    pub native_app: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connect {
    pub sender: MessageSender,
    pub port_id: u64,
    pub extension_id: String,
    pub native_app: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortMessage {
    #[serde(default)]
    pub data: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PortDisconnected {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sender: Option<MessageSender>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PortMessageFromApp {
    pub message: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortDisconnect {
    pub port_id: u64,
}

events! {
    NativeMessage: "GeckoView:WebExtension:Message" => ForEmbedder -> Value;
    Connect: "GeckoView:WebExtension:Connect" => ForEmbedder -> bool;
    PortMessage: "GeckoView:WebExtension:PortMessage" => ForEmbedder;
    PortDisconnected: "GeckoView:WebExtension:Disconnect" => ForEmbedder;
    PortMessageFromApp: "GeckoView:WebExtension:PortMessageFromApp" => ForGecko;
    PortDisconnect: "GeckoView:WebExtension:PortDisconnect" => ForGecko;
}
