use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Voter {
    pub id: i32,
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub password_hash: String,
    pub certificate: Option<String>,
    pub encrypted_private_key: Option<String>,
    pub failed_login_attempts: i32,
    pub certificate_revoked: bool,
    pub created_at: DateTime<Utc>,
}
