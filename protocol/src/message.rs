use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::event::{self, Event, ForEmbedder, ForGecko, Request};

pub const PROTOCOL: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct CallbackId(u64);

impl CallbackId {
    pub const MAX: u64 = (1 << 53) - 1;
    pub const FIRST: Self = Self(1);

    pub const fn new(id: u64) -> Option<Self> {
        if id >= 1 && id <= Self::MAX {
            Some(Self(id))
        } else {
            None
        }
    }

    pub const fn get(self) -> u64 {
        self.0
    }

    pub const fn next(self) -> Option<Self> {
        Self::new(self.0 + 1)
    }
}

impl<'de> Deserialize<'de> for CallbackId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let id = u64::deserialize(deserializer)?;
        Self::new(id).ok_or_else(|| {
            serde::de::Error::custom(format_args!("callback id {id} is outside 1..=2^53-1"))
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct WindowId(String);

impl WindowId {
    pub fn new(id: impl Into<String>) -> Result<Self, TargetError> {
        let id = id.into();
        if id.is_empty() {
            return Err(TargetError::EmptyWindow);
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WindowId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for WindowId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum TargetError {
    #[error("a window id must not be empty")]
    EmptyWindow,
    #[error("a named dispatcher must have a name")]
    EmptyName,
    #[error("`{0}` is not runtime, named:<name> or window:<id>")]
    Unknown(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Target {
    Runtime,
    Named(String),
    Window(WindowId),
}

impl Target {
    const RUNTIME: &'static str = "runtime";
    const NAMED: &'static str = "named:";
    const WINDOW: &'static str = "window:";

    pub fn port(port_id: u64) -> Self {
        Self::Named(format!("port:{port_id}"))
    }

    pub fn window(&self) -> Option<&WindowId> {
        match self {
            Self::Window(window) => Some(window),
            _ => None,
        }
    }
}

impl From<WindowId> for Target {
    fn from(window: WindowId) -> Self {
        Self::Window(window)
    }
}

impl fmt::Display for Target {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Runtime => formatter.write_str(Self::RUNTIME),
            Self::Named(name) => write!(formatter, "{}{name}", Self::NAMED),
            Self::Window(window) => write!(formatter, "{}{window}", Self::WINDOW),
        }
    }
}

impl FromStr for Target {
    type Err = TargetError;

    fn from_str(target: &str) -> Result<Self, Self::Err> {
        if target == Self::RUNTIME {
            Ok(Self::Runtime)
        } else if let Some(name) = target.strip_prefix(Self::NAMED) {
            if name.is_empty() {
                return Err(TargetError::EmptyName);
            }
            Ok(Self::Named(name.to_owned()))
        } else if let Some(window) = target.strip_prefix(Self::WINDOW) {
            WindowId::new(window).map(Self::Window)
        } else {
            Err(TargetError::Unknown(target.to_owned()))
        }
    }
}

impl Serialize for Target {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Target {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?
            .parse()
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventMessage {
    pub target: Target,
    pub event: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callback: Option<CallbackId>,
}

impl EventMessage {
    pub fn new<E: Event>(
        target: Target,
        payload: &E,
        callback: Option<CallbackId>,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            target,
            event: E::NAME.to_owned(),
            data: event::to_data(payload)?,
            callback,
        })
    }

    pub fn is<E: Event>(&self) -> bool {
        self.event == E::NAME
    }

    pub fn payload<E: Event>(&self) -> Result<E, serde_json::Error> {
        event::from_data(self.data.as_ref())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reply {
    pub callback: CallbackId,
    pub success: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl Reply {
    pub fn success<E: Request>(
        callback: CallbackId,
        response: &E::Response,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            callback,
            success: true,
            data: event::to_data(response)?,
        })
    }

    pub fn failure(callback: CallbackId, error: impl Into<Value>) -> Self {
        Self {
            callback,
            success: false,
            data: Some(error.into()),
        }
    }

    pub fn outcome<E: Request>(&self) -> Result<Result<E::Response, Value>, serde_json::Error> {
        if self.success {
            event::from_data(self.data.as_ref()).map(Ok)
        } else {
            Ok(Err(self.data.clone().unwrap_or(Value::Null)))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Release {
    pub callback: CallbackId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Listen {
    pub target: Target,
    pub events: Vec<String>,
}

impl Listen {
    pub fn new(target: Target, events: &[&str]) -> Self {
        Self {
            target,
            events: events.iter().map(|&event| event.to_owned()).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Open {
    pub window: WindowId,
    pub init: Value,
    #[serde(default)]
    pub private: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Close {
    pub window: WindowId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Hello {
    pub protocol: u32,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Closed {
    pub window: WindowId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ready {
    pub window: WindowId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum EmbedderMessage {
    Event(EventMessage),
    Reply(Reply),
    Release(Release),
    Listen(Listen),
    Unlisten(Listen),
    Open(Open),
    Close(Close),
}

impl EmbedderMessage {
    pub fn send<E: ForGecko>(
        target: Target,
        payload: &E,
        callback: Option<CallbackId>,
    ) -> Result<Self, serde_json::Error> {
        EventMessage::new(target, payload, callback).map(Self::Event)
    }

    pub fn answer<E: Request + ForEmbedder>(
        callback: CallbackId,
        response: &E::Response,
    ) -> Result<Self, serde_json::Error> {
        Reply::success::<E>(callback, response).map(Self::Reply)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum GeckoMessage {
    Hello(Hello),
    Event(EventMessage),
    Reply(Reply),
    Release(Release),
    Ready(Ready),
    Closed(Closed),
}
