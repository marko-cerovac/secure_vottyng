use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Vote {
    pub id: i32,
    pub election_id: i32,
    pub voter_id: i32,
    pub encrypted_symmetric_key: String,
    pub encrypted_vote: String,
    pub vote_hmac: String,
    pub signature: String,
    pub cast_at: DateTime<Utc>,
}
