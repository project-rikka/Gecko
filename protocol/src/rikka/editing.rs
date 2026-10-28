use serde::{Deserialize, Serialize};

use crate::geckoview::content::ContextMenu as GeckoContextMenu;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EditCommand {
    Cut,
    Copy,
    Paste,
    PastePlainText,
    SelectAll,
    Undo,
    Redo,
    Delete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Edit {
    pub command: EditCommand,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spelling {
    pub misspelling: String,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextMenu {
    #[serde(flatten)]
    pub menu: GeckoContextMenu,
    pub selection: Option<String>,
    pub editable: bool,
    pub spelling: Option<Spelling>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplaceMisspelling {
    pub suggestion: String,
}

empty_payload! {
    pub struct AddToDictionary;
}

events! {
    Edit: "Rikka:View:Edit" => ForGecko -> ();
    ContextMenu: "Rikka:View:ContextMenu" => ForEmbedder;
    ReplaceMisspelling: "Rikka:View:ReplaceMisspelling" => ForGecko -> ();
    AddToDictionary: "Rikka:View:AddToDictionary" => ForGecko -> ();
}
