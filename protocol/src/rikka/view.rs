use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SetZoom {
    pub zoom: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SnapshotRegion {
    Visible,
    Document,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub region: SnapshotRegion,
    pub scale: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotImage {
    pub width: u32,
    pub height: u32,
    pub png: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrintToPdf {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverLink {
    pub url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tooltip {
    pub x: i32,
    pub y: i32,
    pub text: String,
    pub direction: String,
}

empty_payload! {
    pub struct TooltipHidden;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Icon {
    pub url: String,
    pub rel: String,
    pub sizes: Vec<String>,
    #[serde(rename = "type")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Icons {
    pub page: String,
    pub icons: Vec<Icon>,
}

events! {
    SetZoom: "Rikka:View:SetZoom" => ForGecko -> ();
    Snapshot: "Rikka:View:Snapshot" => ForGecko -> SnapshotImage;
    PrintToPdf: "Rikka:View:SavePdf" => ForGecko -> ();
    OverLink: "Rikka:View:OverLink" => ForEmbedder;
    Tooltip: "Rikka:View:Tooltip" => ForEmbedder;
    TooltipHidden: "Rikka:View:TooltipHidden" => ForEmbedder;
    Icons: "Rikka:View:Icons" => ForEmbedder;
}
