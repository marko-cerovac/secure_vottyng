use postgres::Client;

use crate::models::{AccountRegistrationForm, Organizer, Voter};

pub struct UserService<'a> {
    client: &'a mut Client,
}

impl<'a> UserService<'a> {
    pub fn new(client: &'a mut Client) -> Self {
        UserService { client }
    }

    /// Register a new user (organizer or voter).
    /// TODO: hash password properly
    pub fn register(&mut self, form: AccountRegistrationForm) -> Result<i32, postgres::Error> {
        match form {
            AccountRegistrationForm::Organizer {
                organization,
                password,
            } => {
                let rows = self.client.query(
                    "INSERT INTO organizers (organization, identification_number, password_hash) VALUES ($1, $2, $3) RETURNING id",
                    &[&organization, &organization, &password],
                )?;
                Ok(rows[0].get(0))
            }
            AccountRegistrationForm::User {
                f_name,
                l_name,
                username,
                password,
            } => {
                let rows = self.client.query(
                    "INSERT INTO voters (first_name, last_name, username, password_hash) VALUES ($1, $2, $3, $4) RETURNING id",
                    &[&f_name, &l_name, &username, &password],
                )?;
                Ok(rows[0].get(0))
            }
        }
    }

    /// Fetch a voter by ID.
    pub fn get_voter(&mut self, voter_id: i32) -> Result<Option<Voter>, postgres::Error> {
        let rows = self.client.query(
            "SELECT id, first_name, last_name, username, password_hash, certificate, encrypted_private_key, failed_login_attempts, certificate_revoked, created_at FROM voters WHERE id = $1",
            &[&voter_id],
        )?;
        Ok(rows.first().map(|row| Voter {
            id: row.get(0),
            first_name: row.get(1),
            last_name: row.get(2),
            username: row.get(3),
            password_hash: row.get(4),
            certificate: row.get(5),
            encrypted_private_key: row.get(6),
            failed_login_attempts: row.get(7),
            certificate_revoked: row.get(8),
            created_at: row.get(9),
        }))
    }

    /// Fetch a voter by username.
    pub fn get_voter_by_username(
        &mut self,
        username: &str,
    ) -> Result<Option<Voter>, postgres::Error> {
        let rows = self.client.query(
            "SELECT id, first_name, last_name, username, password_hash, certificate, encrypted_private_key, failed_login_attempts, certificate_revoked, created_at FROM voters WHERE username = $1",
            &[&username],
        )?;
        Ok(rows.first().map(|row| Voter {
            id: row.get(0),
            first_name: row.get(1),
            last_name: row.get(2),
            username: row.get(3),
            password_hash: row.get(4),
            certificate: row.get(5),
            encrypted_private_key: row.get(6),
            failed_login_attempts: row.get(7),
            certificate_revoked: row.get(8),
            created_at: row.get(9),
        }))
    }

    /// Fetch an organizer by ID.
    pub fn get_organizer(
        &mut self,
        organizer_id: i32,
    ) -> Result<Option<Organizer>, postgres::Error> {
        let rows = self.client.query(
            "SELECT id, organization, identification_number, password_hash, certificate, encrypted_private_key, failed_login_attempts, certificate_revoked, created_at FROM organizers WHERE id = $1",
            &[&organizer_id],
        )?;
        Ok(rows.first().map(|row| Organizer {
            id: row.get(0),
            organization: row.get(1),
            identification_number: row.get(2),
            password_hash: row.get(3),
            certificate: row.get(4),
            encrypted_private_key: row.get(5),
            failed_login_attempts: row.get(6),
            certificate_revoked: row.get(7),
            created_at: row.get(8),
        }))
    }

    /// Store a voter's certificate and encrypted private key.
    pub fn store_voter_certificate(
        &mut self,
        voter_id: i32,
        certificate: &str,
        encrypted_private_key: &str,
    ) -> Result<(), postgres::Error> {
        self.client.execute(
            "UPDATE voters SET certificate = $1, encrypted_private_key = $2 WHERE id = $3",
            &[&certificate, &encrypted_private_key, &voter_id],
        )?;
        Ok(())
    }

    /// Store an organizer's certificate and encrypted private key.
    pub fn store_organizer_certificate(
        &mut self,
        organizer_id: i32,
        certificate: &str,
        encrypted_private_key: &str,
    ) -> Result<(), postgres::Error> {
        self.client.execute(
            "UPDATE organizers SET certificate = $1, encrypted_private_key = $2 WHERE id = $3",
            &[&certificate, &encrypted_private_key, &organizer_id],
        )?;
        Ok(())
    }
}
