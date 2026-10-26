use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;

pub trait Event: Serialize + DeserializeOwned {
    const NAME: &'static str;
}

pub trait Request: Event {
    type Response: Serialize + DeserializeOwned;
}

pub trait ForGecko: Event {}

pub trait ForEmbedder: Event {}

pub(crate) fn to_data<T: Serialize>(value: &T) -> Result<Option<Value>, serde_json::Error> {
    match serde_json::to_value(value)? {
        Value::Null => Ok(None),
        data => Ok(Some(data)),
    }
}

pub(crate) fn from_data<T: DeserializeOwned>(data: Option<&Value>) -> Result<T, serde_json::Error> {
    T::deserialize(data.unwrap_or(&Value::Null))
}
