use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Preconnect {
    pub uri: String,
    pub private: bool,
}

events! {
    Preconnect: "Rikka:Network:Preconnect" => ForGecko;
}
