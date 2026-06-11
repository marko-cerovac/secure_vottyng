use chrono::{DateTime, Utc};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum CaType {
    Root,
    Organizational,
    Voter,
}

impl CaType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CaType::Root => "root",
            CaType::Organizational => "organizational",
            CaType::Voter => "voter",
        }
    }
}

impl fmt::Display for CaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct CertificateAuthority {
    pub id: i32,
    pub ca_type: CaType,
    pub certificate: String,
    pub private_key: String,
    pub created_at: DateTime<Utc>,
}
