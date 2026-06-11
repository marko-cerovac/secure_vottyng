use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Organizer {
    pub id: i32,
    pub organization: String,
    pub identification_number: String,
    pub password_hash: String,
    pub certificate: Option<String>,
    pub encrypted_private_key: Option<String>,
    pub failed_login_attempts: i32,
    pub certificate_revoked: bool,
    pub created_at: DateTime<Utc>,
}
