use std::collections::BTreeMap;
use std::fmt::Write;

use serde::{Deserialize, Deserializer, Serialize};

use crate::message::{EmbedderMessage, Open, WindowId};

int_enum! {
    pub enum UserAgentMode: u8 {
        Mobile = 0,
        Desktop = 1,
        Vr = 2,
    }
}

int_enum! {
    pub enum ViewportMode: u8 {
        Mobile = 0,
        Desktop = 1,
    }
}

int_enum! {
    pub enum DisplayMode: u8 {
        Browser = 0,
        MinimalUi = 1,
        Standalone = 2,
        Fullscreen = 3,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct SessionContextId(String);

impl SessionContextId {
    const PREFIX: &'static str = "gvctx";

    pub fn from_unsafe(context: &str) -> Self {
        let bytes = context.as_bytes();
        let negative = bytes.first().is_some_and(|byte| byte & 0x80 != 0);
        let magnitude = if negative {
            twos_complement_magnitude(bytes)
        } else {
            bytes.to_vec()
        };
        let mut id = String::from(Self::PREFIX);
        if negative {
            id.push('-');
        }
        id.push_str(&minimal_hex(&magnitude));
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn is_valid(id: &str) -> bool {
        let Some(rest) = id.strip_prefix(Self::PREFIX) else {
            return false;
        };
        let digits = rest.strip_prefix('-').unwrap_or(rest);
        !digits.is_empty()
            && digits
                .bytes()
                .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
    }
}

impl<'de> Deserialize<'de> for SessionContextId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let id = String::deserialize(deserializer)?;
        if Self::is_valid(&id) {
            Ok(Self(id))
        } else {
            Err(serde::de::Error::custom(format_args!(
                "`{id}` does not match gvctx(-)?[a-f0-9]+"
            )))
        }
    }
}

fn twos_complement_magnitude(bytes: &[u8]) -> Vec<u8> {
    let mut magnitude: Vec<u8> = bytes.iter().map(|byte| !byte).collect();
    for byte in magnitude.iter_mut().rev() {
        let (sum, carry) = byte.overflowing_add(1);
        *byte = sum;
        if !carry {
            break;
        }
    }
    magnitude
}

fn minimal_hex(bytes: &[u8]) -> String {
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(hex, "{byte:02x}").expect("writing to a String cannot fail");
    }
    match hex.trim_start_matches('0') {
        "" => "0".to_owned(),
        digits => digits.to_owned(),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub chrome_uri: Option<String>,
    pub screen_id: i32,
    pub use_tracking_protection: bool,
    pub use_private_mode: bool,
    pub suspend_media_when_inactive: bool,
    pub allow_javascript: bool,
    pub full_accessibility_tree: bool,
    pub is_extension_popup: bool,
    pub user_agent_mode: UserAgentMode,
    pub user_agent_override: Option<String>,
    pub viewport_mode: ViewportMode,
    pub display_mode: DisplayMode,
    session_context_id: Option<SessionContextId>,
    unsafe_session_context_id: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            chrome_uri: None,
            screen_id: 0,
            use_tracking_protection: false,
            use_private_mode: false,
            suspend_media_when_inactive: false,
            allow_javascript: true,
            full_accessibility_tree: false,
            is_extension_popup: false,
            user_agent_mode: UserAgentMode::Mobile,
            user_agent_override: None,
            viewport_mode: ViewportMode::Mobile,
            display_mode: DisplayMode::Browser,
            session_context_id: None,
            unsafe_session_context_id: None,
        }
    }
}

impl Settings {
    pub fn set_context_id(&mut self, context: Option<&str>) {
        self.session_context_id = context.map(SessionContextId::from_unsafe);
        self.unsafe_session_context_id = context.map(str::to_owned);
    }

    pub fn context_id(&self) -> Option<&str> {
        self.unsafe_session_context_id.as_deref()
    }

    pub fn session_context_id(&self) -> Option<&SessionContextId> {
        self.session_context_id.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Module {
    GeckoViewContent,
    GeckoViewNavigation,
    GeckoViewProcessHangMonitor,
    GeckoViewProgress,
    GeckoViewScroll,
    GeckoViewSelectionAction,
    GeckoViewSettings,
    GeckoViewTab,
    GeckoViewContentBlocking,
    GeckoViewAutofill,
    GeckoViewMediaControl,
    GeckoViewAutocomplete,
    GeckoViewPrompter,
    GeckoViewPrintDelegate,
    GeckoViewExperimentDelegate,
    GeckoViewTranslations,
    GeckoViewPageExtractor,
}

impl Module {
    pub const ALL: [Self; 17] = [
        Self::GeckoViewContent,
        Self::GeckoViewNavigation,
        Self::GeckoViewProcessHangMonitor,
        Self::GeckoViewProgress,
        Self::GeckoViewScroll,
        Self::GeckoViewSelectionAction,
        Self::GeckoViewSettings,
        Self::GeckoViewTab,
        Self::GeckoViewContentBlocking,
        Self::GeckoViewAutofill,
        Self::GeckoViewMediaControl,
        Self::GeckoViewAutocomplete,
        Self::GeckoViewPrompter,
        Self::GeckoViewPrintDelegate,
        Self::GeckoViewExperimentDelegate,
        Self::GeckoViewTranslations,
        Self::GeckoViewPageExtractor,
    ];
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InitData {
    pub settings: Settings,
    pub modules: BTreeMap<Module, bool>,
}

impl InitData {
    pub fn new(settings: Settings, enabled: impl IntoIterator<Item = Module>) -> Self {
        let mut modules: BTreeMap<Module, bool> = Module::ALL
            .into_iter()
            .map(|module| (module, false))
            .collect();
        for module in enabled {
            modules.insert(module, true);
        }
        Self { settings, modules }
    }

    pub fn open(&self, window: WindowId) -> Result<EmbedderMessage, serde_json::Error> {
        Ok(EmbedderMessage::Open(Open {
            window,
            init: serde_json::to_value(self)?,
            private: self.settings.use_private_mode,
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct UpdateSettings(pub Settings);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateModuleState {
    pub module: Module,
    pub enabled: bool,
}

events! {
    UpdateSettings: "GeckoView:UpdateSettings" => ForGecko;
    UpdateModuleState: "GeckoView:UpdateModuleState" => ForGecko;
}
