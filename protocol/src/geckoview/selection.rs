use serde::{Deserialize, Serialize};

use crate::geckoview::geometry::{Point, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SelectionAction {
    #[serde(rename = "org.mozilla.geckoview.HIDE")]
    Hide,
    #[serde(rename = "org.mozilla.geckoview.CUT")]
    Cut,
    #[serde(rename = "org.mozilla.geckoview.COPY")]
    Copy,
    #[serde(rename = "org.mozilla.geckoview.PASTE")]
    Paste,
    #[serde(rename = "org.mozilla.geckoview.PASTE_AS_PLAIN_TEXT")]
    PasteAsPlainText,
    #[serde(rename = "org.mozilla.geckoview.DELETE")]
    Delete,
    #[serde(rename = "org.mozilla.geckoview.COLLAPSE_TO_START")]
    CollapseToStart,
    #[serde(rename = "org.mozilla.geckoview.COLLAPSE_TO_END")]
    CollapseToEnd,
    #[serde(rename = "org.mozilla.geckoview.UNSELECT")]
    Unselect,
    #[serde(rename = "org.mozilla.geckoview.SELECT_ALL")]
    SelectAll,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowSelectionAction {
    pub collapsed: bool,
    pub editable: bool,
    pub password: bool,
    #[serde(default)]
    pub selection: Option<String>,
    #[serde(default)]
    pub screen_rect: Option<Rect>,
    pub actions: Vec<SelectionAction>,
    pub action_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HideReason {
    InvisibleSelection,
    PressCaret,
    Scroll,
    VisibilityChange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HideSelectionAction {
    pub reason: HideReason,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteSelectionAction {
    pub id: SelectionAction,
    pub action_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowMagnifier {
    pub screen_point: Point,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipboardPermissionRequest {
    pub screen_point: Point,
}

empty_payload! {
    pub struct HideMagnifier;
    pub struct DismissClipboardPermissionRequest;
}

events! {
    ShowSelectionAction: "GeckoView:ShowSelectionAction" => ForEmbedder;
    HideSelectionAction: "GeckoView:HideSelectionAction" => ForEmbedder;
    ShowMagnifier: "GeckoView:ShowMagnifier" => ForEmbedder;
    HideMagnifier: "GeckoView:HideMagnifier" => ForEmbedder;
    ClipboardPermissionRequest: "GeckoView:ClipboardPermissionRequest" => ForEmbedder -> bool;
    DismissClipboardPermissionRequest: "GeckoView:DismissClipboardPermissionRequest" => ForEmbedder -> ();
    ExecuteSelectionAction: "GeckoView:ExecuteSelectionAction" => ForGecko;
}
