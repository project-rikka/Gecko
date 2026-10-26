use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionHistory {
    #[serde(default)]
    pub entries: Vec<HistoryEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_index: Option<u32>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryChange {
    pub from_idx: i64,
    #[serde(flatten)]
    pub history: SessionHistory,
}

impl HistoryChange {
    pub const FULL: i64 = -1;
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SessionStateUpdate {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub historychange: Option<HistoryChange>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scroll: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formdata: Option<Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateUpdated {
    pub data: SessionStateUpdate,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SessionState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history: Option<SessionHistory>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scrolldata: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formdata: Option<Value>,
}

impl SessionState {
    pub fn apply(&mut self, update: SessionStateUpdate) {
        if let Some(change) = update.historychange {
            self.history = Some(self.merge_history(change));
        }
        if let Some(scroll) = update.scroll {
            self.scrolldata = Some(scroll);
        }
        if let Some(formdata) = update.formdata {
            self.formdata = Some(formdata);
        }
    }

    pub fn entries(&self) -> &[HistoryEntry] {
        self.history
            .as_ref()
            .map_or(&[], |history| &history.entries)
    }

    pub fn current_index(&self) -> Option<usize> {
        let index = self.history.as_ref()?.index?;
        usize::try_from(index).ok()?.checked_sub(1)
    }

    pub fn is_empty(&self) -> bool {
        self.history.is_none() && self.scrolldata.is_none() && self.formdata.is_none()
    }

    fn merge_history(&mut self, change: HistoryChange) -> SessionHistory {
        let HistoryChange {
            from_idx,
            history: mut incoming,
        } = change;
        if from_idx == HistoryChange::FULL {
            return incoming;
        }
        let previous = self
            .history
            .take()
            .map(|history| history.entries)
            .unwrap_or_default();
        let kept = usize::try_from(from_idx.saturating_add(1)).unwrap_or(0);
        incoming.entries = previous
            .into_iter()
            .take(kept)
            .chain(incoming.entries)
            .collect();
        incoming
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RestoreState(pub SessionState);

empty_payload! {
    pub struct FlushSessionState;
    pub struct ContainsFormData;
}

events! {
    StateUpdated: "GeckoView:StateUpdated" => ForEmbedder;
    FlushSessionState: "GeckoView:FlushSessionState" => ForGecko;
    RestoreState: "GeckoView:RestoreState" => ForGecko;
    ContainsFormData: "GeckoView:ContainsFormData" => ForGecko -> bool;
}
