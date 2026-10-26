use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PrefValue {
    Bool(bool),
    Int(i32),
    String(String),
}

impl PrefValue {
    pub const fn pref_type(&self) -> PrefType {
        match self {
            Self::Bool(_) => PrefType::Bool,
            Self::Int(_) => PrefType::Int,
            Self::String(_) => PrefType::String,
        }
    }
}

int_enum! {
    pub enum PrefType: u8 {
        Invalid = 0,
        String = 32,
        Int = 64,
        Bool = 128,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrefBranch {
    User,
    Default,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preference {
    pub pref: String,
    #[serde(rename = "type")]
    pub pref_type: PrefType,
    pub default_value: Option<PrefValue>,
    pub user_value: Option<PrefValue>,
}

impl Preference {
    pub fn value(&self) -> Option<&PrefValue> {
        self.user_value.as_ref().or(self.default_value.as_ref())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefAssignment {
    pub pref: String,
    pub value: PrefValue,
    pub branch: PrefBranch,
}

#[derive(Serialize, Deserialize)]
struct PrefAssignmentWire {
    pref: String,
    #[serde(rename = "type")]
    pref_type: PrefType,
    value: PrefValue,
    branch: PrefBranch,
}

impl Serialize for PrefAssignment {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        PrefAssignmentWire {
            pref: self.pref.clone(),
            pref_type: self.value.pref_type(),
            value: self.value.clone(),
            branch: self.branch,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for PrefAssignment {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = PrefAssignmentWire::deserialize(deserializer)?;
        if wire.pref_type != wire.value.pref_type() {
            return Err(serde::de::Error::custom(format_args!(
                "{} is declared {:?} but holds {:?}",
                wire.pref,
                wire.pref_type,
                wire.value.pref_type()
            )));
        }
        Ok(Self {
            pref: wire.pref,
            value: wire.value,
            branch: wire.branch,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetPrefs {
    pub prefs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preferences {
    pub prefs: Vec<Preference>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetPrefs {
    pub prefs: Vec<PrefAssignment>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrefSetResult {
    pub pref: String,
    pub is_set: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrefSetResults {
    pub prefs: Vec<PrefSetResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClearPref {
    pub pref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservePrefs {
    pub prefs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnobservePrefs {
    pub prefs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreferenceChanged {
    pub data: Preference,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SetDefaultPrefs(pub BTreeMap<String, PrefValue>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResetUserPrefs {
    pub names: Vec<String>,
}

events! {
    GetPrefs: "GeckoView:Preferences:GetPref" => ForGecko -> Preferences;
    SetPrefs: "GeckoView:Preferences:SetPref" => ForGecko -> PrefSetResults;
    ClearPref: "GeckoView:Preferences:ClearPref" => ForGecko -> ();
    ObservePrefs: "GeckoView:Preferences:RegisterObserver" => ForGecko -> ();
    UnobservePrefs: "GeckoView:Preferences:UnregisterObserver" => ForGecko -> ();
    PreferenceChanged: "GeckoView:GeckoPreferences:Change" => ForEmbedder;
    SetDefaultPrefs: "GeckoView:SetDefaultPrefs" => ForGecko;
    ResetUserPrefs: "GeckoView:ResetUserPrefs" => ForGecko;
}
