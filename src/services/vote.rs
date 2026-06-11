use postgres::Client;

use crate::models::Vote;

pub struct VoteService<'a> {
    client: &'a mut Client,
}

impl<'a> VoteService<'a> {
    pub fn new(client: &'a mut Client) -> Self {
        VoteService { client }
    }

    /// Cast a vote. Returns the vote ID.
    pub fn cast(
        &mut self,
        election_id: i32,
        voter_id: i32,
        encrypted_symmetric_key: &str,
        encrypted_vote: &str,
        vote_hmac: &str,
        signature: &str,
    ) -> Result<i32, postgres::Error> {
        let rows = self.client.query(
            "INSERT INTO votes (election_id, voter_id, encrypted_symmetric_key, encrypted_vote, vote_hmac, signature) VALUES ($1, $2, $3, $4, $5, $6) RETURNING id",
            &[&election_id, &voter_id, &encrypted_symmetric_key, &encrypted_vote, &vote_hmac, &signature],
        )?;
        Ok(rows[0].get(0))
    }

    /// Check whether a voter has already voted in an election.
    pub fn has_voted(&mut self, election_id: i32, voter_id: i32) -> Result<bool, postgres::Error> {
        let rows = self.client.query(
            "SELECT id FROM votes WHERE election_id = $1 AND voter_id = $2",
            &[&election_id, &voter_id],
        )?;
        Ok(!rows.is_empty())
    }

    /// Fetch a voter's own vote for verification.
    pub fn get_by_voter(
        &mut self,
        election_id: i32,
        voter_id: i32,
    ) -> Result<Option<Vote>, postgres::Error> {
        let rows = self.client.query(
            "SELECT id, election_id, voter_id, encrypted_symmetric_key, encrypted_vote, vote_hmac, signature, cast_at FROM votes WHERE election_id = $1 AND voter_id = $2",
            &[&election_id, &voter_id],
        )?;
        Ok(rows.first().map(|row| row_to_vote(row)))
    }

    /// Fetch all votes for an election (used during counting by the organizer).
    pub fn get_all_for_election(&mut self, election_id: i32) -> Result<Vec<Vote>, postgres::Error> {
        let rows = self.client.query(
            "SELECT id, election_id, voter_id, encrypted_symmetric_key, encrypted_vote, vote_hmac, signature, cast_at FROM votes WHERE election_id = $1 ORDER BY cast_at ASC",
            &[&election_id],
        )?;
        Ok(rows.iter().map(|row| row_to_vote(row)).collect())
    }
}

fn row_to_vote(row: &postgres::Row) -> Vote {
    Vote {
        id: row.get(0),
        election_id: row.get(1),
        voter_id: row.get(2),
        encrypted_symmetric_key: row.get(3),
        encrypted_vote: row.get(4),
        vote_hmac: row.get(5),
        signature: row.get(6),
        cast_at: row.get(7),
    }
}
