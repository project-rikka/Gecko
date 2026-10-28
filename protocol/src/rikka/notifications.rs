use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationAction {
    pub action: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowNotification {
    pub name: String,
    pub title: String,
    pub text: String,
    pub image_url: Option<String>,
    pub origin: Option<String>,
    pub lang: Option<String>,
    pub dir: Option<String>,
    pub silent: bool,
    pub require_interaction: bool,
    pub private: bool,
    pub actions: Vec<NotificationAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CloseNotification {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationClicked {
    pub name: String,
    pub action: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotificationClosed {
    pub name: String,
}

events! {
    ShowNotification: "Rikka:Notification:Show" => ForEmbedder;
    CloseNotification: "Rikka:Notification:Close" => ForEmbedder;
    NotificationClicked: "Rikka:Notification:Clicked" => ForGecko;
    NotificationClosed: "Rikka:Notification:Closed" => ForGecko;
}
