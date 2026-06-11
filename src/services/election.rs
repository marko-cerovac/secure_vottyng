use chrono::{DateTime, Utc};
use postgres::Client;

use crate::models::{Candidate, Election, ElectionStatus};

pub struct ElectionService<'a> {
    client: &'a mut Client,
}

impl<'a> ElectionService<'a> {
    pub fn new(client: &'a mut Client) -> Self {
        ElectionService { client }
    }

    /// Create a new election with its candidates. Returns the election ID.
    pub fn create(
        &mut self,
        organizer_id: i32,
        title: &str,
        description: &str,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
        candidate_names: &[String],
    ) -> Result<i32, postgres::Error> {
        let rows = self.client.query(
            "INSERT INTO elections (organizer_id, title, description, starts_at, ends_at) VALUES ($1, $2, $3, $4, $5) RETURNING id",
            &[&organizer_id, &title, &description, &starts_at, &ends_at],
        )?;
        let election_id: i32 = rows[0].get(0);

        for (i, name) in candidate_names.iter().enumerate() {
            let position = i as i32;
            self.client.execute(
                "INSERT INTO candidates (election_id, name, position) VALUES ($1, $2, $3)",
                &[&election_id, &name, &position],
            )?;
        }

        Ok(election_id)
    }

    /// List all elections.
    pub fn list_all(&mut self) -> Result<Vec<Election>, postgres::Error> {
        let rows = self.client.query(
            "SELECT id, organizer_id, title, description, starts_at, ends_at, status, results_report, results_signature, created_at FROM elections ORDER BY created_at DESC",
            &[],
        )?;
        Ok(rows.iter().map(|row| row_to_election(row)).collect())
    }

    /// List elections by organizer.
    pub fn list_by_organizer(
        &mut self,
        organizer_id: i32,
    ) -> Result<Vec<Election>, postgres::Error> {
        let rows = self.client.query(
            "SELECT id, organizer_id, title, description, starts_at, ends_at, status, results_report, results_signature, created_at FROM elections WHERE organizer_id = $1 ORDER BY created_at DESC",
            &[&organizer_id],
        )?;
        Ok(rows.iter().map(|row| row_to_election(row)).collect())
    }

    /// List active elections (available for voting).
    pub fn list_active(&mut self) -> Result<Vec<Election>, postgres::Error> {
        let rows = self.client.query(
            "SELECT id, organizer_id, title, description, starts_at, ends_at, status, results_report, results_signature, created_at FROM elections WHERE status = 'active' ORDER BY ends_at ASC",
            &[],
        )?;
        Ok(rows.iter().map(|row| row_to_election(row)).collect())
    }

    /// Fetch a single election by ID.
    pub fn get_by_id(&mut self, election_id: i32) -> Result<Option<Election>, postgres::Error> {
        let rows = self.client.query(
            "SELECT id, organizer_id, title, description, starts_at, ends_at, status, results_report, results_signature, created_at FROM elections WHERE id = $1",
            &[&election_id],
        )?;
        Ok(rows.first().map(|row| row_to_election(row)))
    }

    /// Get the candidates for an election.
    pub fn get_candidates(&mut self, election_id: i32) -> Result<Vec<Candidate>, postgres::Error> {
        let rows = self.client.query(
            "SELECT id, election_id, name, position FROM candidates WHERE election_id = $1 ORDER BY position ASC",
            &[&election_id],
        )?;
        Ok(rows
            .iter()
            .map(|row| Candidate {
                id: row.get(0),
                election_id: row.get(1),
                name: row.get(2),
                position: row.get(3),
            })
            .collect())
    }

    /// Update election status (e.g. pending -> active -> closed -> counted).
    pub fn update_status(
        &mut self,
        election_id: i32,
        status: &ElectionStatus,
    ) -> Result<(), postgres::Error> {
        self.client.execute(
            "UPDATE elections SET status = $1 WHERE id = $2",
            &[&status.as_str(), &election_id],
        )?;
        Ok(())
    }

    /// Store the signed results report after vote counting.
    pub fn store_results(
        &mut self,
        election_id: i32,
        report: &str,
        signature: &str,
    ) -> Result<(), postgres::Error> {
        self.client.execute(
            "UPDATE elections SET results_report = $1, results_signature = $2, status = 'counted' WHERE id = $3",
            &[&report, &signature, &election_id],
        )?;
        Ok(())
    }
}

fn row_to_election(row: &postgres::Row) -> Election {
    let status_str: String = row.get(6);
    let status = match status_str.as_str() {
        "active" => ElectionStatus::Active,
        "closed" => ElectionStatus::Closed,
        "counted" => ElectionStatus::Counted,
        _ => ElectionStatus::Pending,
    };
    Election {
        id: row.get(0),
        organizer_id: row.get(1),
        title: row.get(2),
        description: row.get(3),
        starts_at: row.get(4),
        ends_at: row.get(5),
        status,
        results_report: row.get(7),
        results_signature: row.get(8),
        created_at: row.get(9),
    }
}
