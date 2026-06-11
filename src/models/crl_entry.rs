use chrono::{DateTime, Utc};

use super::CaType;

#[derive(Debug, Clone)]
pub struct CrlEntry {
    pub id: i32,
    pub ca_type: CaType,
    pub serial_number: String,
    pub revoked_at: DateTime<Utc>,
    pub reason: Option<String>,
}
