use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LoginEntry {
    pub guid: Option<String>,
    pub origin: String,
    pub form_action_origin: Option<String>,
    pub http_realm: Option<String>,
    pub username: String,
    pub password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_last_used: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_password_changed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub times_used: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Address {
    pub guid: Option<String>,
    pub name: String,
    pub given_name: String,
    pub additional_name: String,
    pub family_name: String,
    pub organization: String,
    pub street_address: String,
    pub address_level1: String,
    pub address_level2: String,
    pub address_level3: String,
    pub postal_code: String,
    pub country: String,
    pub tel: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_last_used: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_last_modified: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub times_used: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CreditCard {
    pub guid: Option<String>,
    pub name: String,
    pub number: String,
    pub exp_month: String,
    pub exp_year: String,
    #[serde(rename = "type")]
    pub card_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_created: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_last_used: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_last_modified: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub times_used: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
}

flags! {
    pub struct SelectHint: u32 {
        const GENERATED = 1 << 0;
        const INSECURE_FORM = 1 << 1;
        const DUPLICATE_USERNAME = 1 << 2;
        const MATCHING_ORIGIN = 1 << 3;
        const FIREFOX_RELAY = 1 << 4;
    }
}

flags! {
    pub struct LoginSaveHint: u32 {
        const GENERATED = 1 << 0;
        const LOW_CONFIDENCE = 1 << 1;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectOption<T> {
    pub value: T,
    #[serde(default)]
    pub hint: SelectHint,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FetchLogins {
    pub domain: Option<String>,
}

empty_payload! {
    pub struct FetchCreditCards;
    pub struct FetchAddresses;
    pub struct StorageDelegateAttached;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveLogin {
    pub login: LoginEntry,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveCreditCard {
    pub credit_card: CreditCard,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SaveAddress {
    pub address: Address,
}

flags! {
    pub struct UsedFields: u32 {
        const PASSWORD = 1 << 0;
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginUsed {
    pub used_fields: UsedFields,
    pub login: LoginEntry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GetAddressStructure {
    pub country: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddressFieldOption {
    pub text: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressField {
    pub field_id: String,
    pub l10n_id: String,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub options: Option<Vec<AddressFieldOption>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddressStructure {
    pub fields: Vec<AddressField>,
}

events! {
    FetchLogins: "GeckoView:Autocomplete:Fetch:Login" => ForEmbedder -> Vec<LoginEntry>;
    FetchCreditCards: "GeckoView:Autocomplete:Fetch:CreditCard" => ForEmbedder -> Vec<CreditCard>;
    FetchAddresses: "GeckoView:Autocomplete:Fetch:Address" => ForEmbedder -> Vec<Address>;
    SaveLogin: "GeckoView:Autocomplete:Save:Login" => ForEmbedder;
    SaveCreditCard: "GeckoView:Autocomplete:Save:CreditCard" => ForEmbedder;
    SaveAddress: "GeckoView:Autocomplete:Save:Address" => ForEmbedder;
    LoginUsed: "GeckoView:Autocomplete:Used:Login" => ForEmbedder;
    StorageDelegateAttached: "GeckoView:StorageDelegate:Attached" => ForGecko;
    GetAddressStructure: "GeckoView:Autofill:GetAddressStructure" => ForGecko -> AddressStructure;
}
