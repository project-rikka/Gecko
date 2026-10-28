use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DotPrintRequest {
    pub canonical_browsing_context_id: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DotPrintFinish {
    pub is_pdf_successful: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_reason: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePdf {
    pub url: String,
    pub filename: String,
    pub original_url: String,
    pub is_private: bool,
    pub skip_confirmation: bool,
    pub request_external_app: bool,
}

empty_payload! {
    pub struct PdfSave;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedPdf {
    pub url: String,
    pub filename: String,
    pub original_url: String,
    pub is_private: bool,
}

events! {
    DotPrintRequest: "GeckoView:DotPrintRequest" => ForEmbedder;
    DotPrintFinish: "GeckoView:DotPrintFinish" => ForGecko;
    SavePdf: "GeckoView:SavePdf" => ForEmbedder;
    PdfSave: "GeckoView:PDFSave" => ForGecko -> SavedPdf;
}
