use chrono::{DateTime, Utc};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum ElectionStatus {
    Pending,
    Active,
    Closed,
    Counted,
}

impl ElectionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ElectionStatus::Pending => "pending",
            ElectionStatus::Active => "active",
            ElectionStatus::Closed => "closed",
            ElectionStatus::Counted => "counted",
        }
    }
}

impl fmt::Display for ElectionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Election {
    pub id: i32,
    pub organizer_id: i32,
    pub title: String,
    pub description: String,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub status: ElectionStatus,
    pub results_report: Option<String>,
    pub results_signature: Option<String>,
    pub created_at: DateTime<Utc>,
}
