use serde::{Deserialize, Serialize};

use crate::geckoview::autocomplete::{
    Address, CreditCard, LoginEntry, LoginSaveHint, SelectHint, SelectOption,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PromptId(pub String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Prompt {
    pub id: PromptId,
    #[serde(flatten)]
    pub kind: PromptKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PromptKind {
    #[serde(rename = "alert")]
    Alert(AlertPrompt),
    #[serde(rename = "button")]
    Button(ButtonPrompt),
    #[serde(rename = "text")]
    Text(TextPrompt),
    #[serde(rename = "auth")]
    Auth(AuthPrompt),
    #[serde(rename = "choice")]
    Choice(ChoicePrompt),
    #[serde(rename = "color")]
    Color(ColorPrompt),
    #[serde(rename = "datetime")]
    DateTime(DateTimePrompt),
    #[serde(rename = "file")]
    File(FilePrompt),
    #[serde(rename = "folderUpload")]
    FolderUpload(FolderUploadPrompt),
    #[serde(rename = "popup")]
    Popup(TargetPrompt),
    #[serde(rename = "redirect")]
    Redirect(TargetPrompt),
    #[serde(rename = "repost")]
    Repost {},
    #[serde(rename = "beforeUnload")]
    BeforeUnload {},
    #[serde(rename = "share")]
    Share(SharePrompt),
    #[serde(rename = "Autocomplete:Save:Login")]
    LoginSave(LoginSavePrompt),
    #[serde(rename = "Autocomplete:Select:Login")]
    LoginSelect(SelectPrompt<LoginEntry>),
    #[serde(rename = "Autocomplete:Select:CreditCard")]
    CreditCardSelect(SelectPrompt<CreditCard>),
    #[serde(rename = "Autocomplete:Select:Address")]
    AddressSelect(SelectPrompt<Address>),
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptCheck {
    #[serde(default)]
    pub has_check: bool,
    #[serde(default)]
    pub check_msg: Option<String>,
    #[serde(default)]
    pub check_value: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlertPrompt {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(flatten)]
    pub check: PromptCheck,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ButtonTitle {
    Ok,
    Cancel,
    Yes,
    No,
    Custom,
}

int_enum! {
    pub enum ButtonType: u8 {
        Positive = 0,
        Neutral = 1,
        Negative = 2,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ButtonPrompt {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub msg: Option<String>,
    pub btn_title: [Option<ButtonTitle>; 3],
    pub btn_custom_title: [Option<String>; 3],
    #[serde(flatten)]
    pub check: PromptCheck,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextPrompt {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub msg: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(flatten)]
    pub check: PromptCheck,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthMode {
    Auth,
    Password,
}

flags! {
    pub struct AuthFlags: u32 {
        const HOST = 1 << 0;
        const PROXY = 1 << 1;
        const ONLY_PASSWORD = 1 << 3;
        const PREVIOUS_FAILED = 1 << 4;
        const CROSS_ORIGIN_SUB_RESOURCE = 1 << 5;
        const NEED_DOMAIN = 1 << 2;
    }
}

int_enum! {
    pub enum AuthLevel: u8 {
        None = 0,
        PasswordEncrypted = 1,
        Secure = 2,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthOptions {
    pub flags: AuthFlags,
    #[serde(default)]
    pub uri: Option<String>,
    #[serde(default)]
    pub level: Option<AuthLevel>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthPrompt {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub msg: Option<String>,
    pub mode: AuthMode,
    pub options: AuthOptions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChoiceMode {
    Menu,
    Single,
    Multiple,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choice {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub selected: bool,
    #[serde(default)]
    pub separator: bool,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub items: Option<Vec<Choice>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoicePrompt {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub msg: Option<String>,
    pub mode: ChoiceMode,
    pub choices: Vec<Choice>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ColorPrompt {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub predefined_values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DateTimeMode {
    Date,
    Month,
    Week,
    Time,
    DatetimeLocal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateTimePrompt {
    #[serde(default)]
    pub title: Option<String>,
    pub mode: DateTimeMode,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub min: Option<String>,
    #[serde(default)]
    pub max: Option<String>,
    #[serde(default)]
    pub step: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileMode {
    Single,
    Multiple,
    Folder,
}

int_enum! {
    pub enum CaptureType: u8 {
        None = 0,
        Any = 1,
        User = 2,
        Environment = 3,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilePrompt {
    #[serde(default)]
    pub title: Option<String>,
    pub mode: FileMode,
    #[serde(default)]
    pub mime_types: Vec<String>,
    pub capture: CaptureType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FolderUploadPrompt {
    pub directory_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TargetPrompt {
    #[serde(default)]
    pub target_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SharePrompt {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LoginSavePrompt {
    pub hint: LoginSaveHint,
    pub logins: Vec<LoginEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectPrompt<T> {
    pub options: Vec<SelectOption<T>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CheckResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_value: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ButtonResponse {
    pub button: ButtonType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_value: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TextResponse {
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_value: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthResponse {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChoiceResponse {
    pub choices: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ColorResponse {
    pub color: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DateTimeResponse {
    pub datetime: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryFile {
    pub file_path: String,
    pub relative_path: String,
    #[serde(rename = "type")]
    pub mime_type: String,
    pub last_modified: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct FileResponse {
    pub files: Vec<String>,
    #[serde(
        default,
        rename = "filesInWebKitDirectory",
        skip_serializing_if = "Option::is_none"
    )]
    pub files_in_webkit_directory: Option<Vec<DirectoryFile>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AllowResponse {
    pub allow: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetResponse {
    pub response: bool,
}

int_enum! {
    pub enum ShareResult: u8 {
        Success = 0,
        Failure = 1,
        Abort = 2,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShareResponse {
    pub response: ShareResult,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectionResponse<T> {
    pub selection: Selection<T>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection<T> {
    pub value: T,
    pub hint: SelectHint,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PromptResponse {
    Button(ButtonResponse),
    Text(TextResponse),
    Auth(AuthResponse),
    Choice(ChoiceResponse),
    Color(ColorResponse),
    DateTime(DateTimeResponse),
    File(FileResponse),
    Allow(AllowResponse),
    Target(TargetResponse),
    Share(ShareResponse),
    Login(SelectionResponse<LoginEntry>),
    CreditCard(SelectionResponse<CreditCard>),
    Address(SelectionResponse<Address>),
    Check(CheckResponse),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShowPrompt {
    pub prompt: Prompt,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdatePrompt {
    pub prompt: Prompt,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DismissPrompt {
    pub id: PromptId,
}

events! {
    ShowPrompt: "GeckoView:Prompt" => ForEmbedder -> Option<PromptResponse>;
    UpdatePrompt: "GeckoView:Prompt:Update" => ForEmbedder;
    DismissPrompt: "GeckoView:Prompt:Dismiss" => ForEmbedder;
}
