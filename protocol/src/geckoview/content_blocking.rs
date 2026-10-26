use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentBlockingEvent {
    pub uri: String,
    pub category: u32,
    #[serde(default)]
    pub blocked_list: Option<String>,
    #[serde(default)]
    pub loaded_lists: Vec<String>,
}

empty_payload! {
    pub struct RequestLog;
    pub struct SumAllEvents;
    pub struct GetEarliestRecordedDate;
    pub struct ClearTrackingDb;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockData {
    pub category: u32,
    pub blocked: bool,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub origin: String,
    pub block_data: Vec<BlockData>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentBlockingLog {
    pub log: Vec<LogEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetEventsByDateRange {
    pub date_from: i64,
    pub date_to: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackingEvent {
    #[serde(rename = "type")]
    pub event_type: u32,
    pub count: u32,
    pub date: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackingEvents {
    pub events: Vec<TrackingEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackingSum {
    pub sum: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EarliestRecordedDate {
    pub date: i64,
}

events! {
    ContentBlockingEvent: "GeckoView:ContentBlockingEvent" => ForEmbedder;
    RequestLog: "ContentBlocking:RequestLog" => ForGecko -> ContentBlockingLog;
    GetEventsByDateRange: "GeckoView:TrackingDB:GetEventsByDateRange" => ForGecko -> TrackingEvents;
    SumAllEvents: "GeckoView:TrackingDB:SumAllEvents" => ForGecko -> TrackingSum;
    GetEarliestRecordedDate: "GeckoView:TrackingDB:GetEarliestRecordedDate" => ForGecko -> EarliestRecordedDate;
    ClearTrackingDb: "GeckoView:TrackingDB:ClearAll" => ForGecko -> ();
}
