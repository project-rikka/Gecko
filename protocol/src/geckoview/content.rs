use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PageTitleChanged {
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewImage {
    pub preview_image_url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebAppManifest {
    pub manifest: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetaViewportFit {
    pub viewportfit: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextMenu {
    pub screen_x: f64,
    pub screen_y: f64,
    #[serde(default)]
    pub base_uri: Option<String>,
    #[serde(default)]
    pub uri: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub alt: Option<String>,
    pub element_type: String,
    #[serde(default)]
    pub element_src: Option<String>,
    #[serde(default)]
    pub text_content: Option<String>,
    #[serde(default)]
    pub link_text: Option<String>,
}

empty_payload! {
    pub struct WindowCloseRequest;
    pub struct FocusRequest;
    pub struct ContentCrash;
    pub struct ContentKill;
    pub struct FirstContentfulPaint;
    pub struct PaintStatusReset;
    pub struct ExitFullScreen;
    pub struct GetUserAgent;
    pub struct IsPdfJs;
    pub struct ProcessBackPressed;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetActive {
    pub active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SetFocused {
    pub focused: bool,
}

int_enum! {
    pub enum Priority: u8 {
        Default = 0,
        High = 1,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetPriorityHint {
    pub priority_hint: Priority,
}

int_enum! {
    pub enum ScreenLengthType: u8 {
        Pixel = 0,
        VisualViewportWidth = 1,
        VisualViewportHeight = 2,
        DocumentWidth = 3,
        DocumentHeight = 4,
    }
}

int_enum! {
    pub enum ScrollBehavior: u8 {
        Smooth = 0,
        Auto = 1,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrollRequest {
    pub width_value: f64,
    pub width_type: ScreenLengthType,
    pub height_value: f64,
    pub height_type: ScreenLengthType,
    pub behavior: ScrollBehavior,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ScrollBy(pub ScrollRequest);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ScrollTo(pub ScrollRequest);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScrollChanged {
    pub scroll_x: f64,
    pub scroll_y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinOnScreen {
    pub pinned: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HangReport {
    pub hang_id: u32,
    pub script_file_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HangReportStop {
    pub hang_id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HangReportWait {
    pub hang_id: u32,
}

events! {
    PageTitleChanged: "GeckoView:PageTitleChanged" => ForEmbedder;
    PreviewImage: "GeckoView:PreviewImage" => ForEmbedder;
    WebAppManifest: "GeckoView:WebAppManifest" => ForEmbedder;
    MetaViewportFit: "GeckoView:DOMMetaViewportFit" => ForEmbedder;
    ContextMenu: "GeckoView:ContextMenu" => ForEmbedder;
    WindowCloseRequest: "GeckoView:DOMWindowClose" => ForEmbedder;
    FocusRequest: "GeckoView:FocusRequest" => ForEmbedder;
    ContentCrash: "GeckoView:ContentCrash" => ForEmbedder;
    ContentKill: "GeckoView:ContentKill" => ForEmbedder;
    FirstContentfulPaint: "GeckoView:FirstContentfulPaint" => ForEmbedder;
    PaintStatusReset: "GeckoView:PaintStatusReset" => ForEmbedder;
    ScrollChanged: "GeckoView:ScrollChanged" => ForEmbedder;
    PinOnScreen: "GeckoView:PinOnScreen" => ForEmbedder;
    HangReport: "GeckoView:HangReport" => ForEmbedder;
    ExitFullScreen: "GeckoViewContent:ExitFullScreen" => ForGecko;
    SetActive: "GeckoView:SetActive" => ForGecko;
    SetFocused: "GeckoView:SetFocused" => ForGecko;
    SetPriorityHint: "GeckoView:SetPriorityHint" => ForGecko;
    ScrollBy: "GeckoView:ScrollBy" => ForGecko;
    ScrollTo: "GeckoView:ScrollTo" => ForGecko;
    HangReportStop: "GeckoView:HangReportStop" => ForGecko;
    HangReportWait: "GeckoView:HangReportWait" => ForGecko;
    GetUserAgent: "GeckoView:GetUserAgent" => ForGecko -> String;
    IsPdfJs: "GeckoView:IsPdfJs" => ForGecko -> bool;
    ProcessBackPressed: "GeckoView:ProcessBackPressed" => ForGecko -> bool;
}
