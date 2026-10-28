use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InjectAt {
    Start,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Frames {
    All,
    Top,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StyleLevel {
    User,
    Author,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserScript {
    pub id: String,
    pub source: String,
    pub inject_at: InjectAt,
    pub frames: Frames,
    pub world: Option<String>,
    pub allow: Vec<String>,
    pub block: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserStyle {
    pub id: String,
    pub source: String,
    pub frames: Frames,
    pub level: StyleLevel,
    pub allow: Vec<String>,
    pub block: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageHandler {
    pub name: String,
    pub world: Option<String>,
    pub replying: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SetUserContent {
    pub scripts: Vec<UserScript>,
    pub styles: Vec<UserStyle>,
    pub handlers: Vec<MessageHandler>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evaluate {
    pub world: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CallFunction {
    pub world: Option<String>,
    pub body: String,
    pub arguments: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageFrame {
    pub url: String,
    pub top: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScriptMessage {
    pub name: String,
    pub world: Option<String>,
    pub body: Value,
    pub frame: MessageFrame,
}

events! {
    SetUserContent: "Rikka:UserContent:Set" => ForGecko;
    Evaluate: "Rikka:UserContent:Evaluate" => ForGecko -> Value;
    CallFunction: "Rikka:UserContent:Call" => ForGecko -> Value;
    ScriptMessage: "Rikka:UserContent:Message" => ForEmbedder -> Value;
}
