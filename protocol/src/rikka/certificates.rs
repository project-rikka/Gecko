use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateError {
    pub uri: String,
    pub host: String,
    pub port: u16,
    pub certificate: String,
    pub overridable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AllowCertificate {
    pub host: String,
    pub port: u16,
    pub certificate: String,
    pub permanent: bool,
}

events! {
    CertificateError: "Rikka:View:CertificateError" => ForEmbedder;
    AllowCertificate: "Rikka:View:AllowCertificate" => ForGecko -> ();
}
