use postgres::Client;

pub struct AuthService<'a> {
    client: &'a mut Client,
}

impl<'a> AuthService<'a> {
    pub fn new(client: &'a mut Client) -> Self {
        AuthService { client }
    }

    /// Find a voter by their certificate PEM. Returns the voter's ID if found.
    pub fn find_voter_by_cert(&mut self, cert_pem: &str) -> Result<Option<i32>, postgres::Error> {
        let rows = self.client.query(
            "SELECT id, certificate_revoked FROM voters WHERE certificate = $1",
            &[&cert_pem],
        )?;
        match rows.first() {
            Some(row) => {
                let revoked: bool = row.get(1);
                if revoked {
                    Ok(None)
                } else {
                    Ok(Some(row.get(0)))
                }
            }
            None => Ok(None),
        }
    }

    /// Find an organizer by their certificate PEM. Returns the organizer's ID if found.
    pub fn find_organizer_by_cert(
        &mut self,
        cert_pem: &str,
    ) -> Result<Option<i32>, postgres::Error> {
        let rows = self.client.query(
            "SELECT id, certificate_revoked FROM organizers WHERE certificate = $1",
            &[&cert_pem],
        )?;
        match rows.first() {
            Some(row) => {
                let revoked: bool = row.get(1);
                if revoked {
                    Ok(None)
                } else {
                    Ok(Some(row.get(0)))
                }
            }
            None => Ok(None),
        }
    }

    /// Authenticate a voter by username and password.
    pub fn authenticate_voter(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<bool, postgres::Error> {
        let rows = self.client.query(
            "SELECT id FROM voters WHERE username = $1 AND password_hash = $2",
            &[&username, &password],
        )?;
        Ok(!rows.is_empty())
    }

    /// Authenticate an organizer by identification number and password.
    pub fn authenticate_organizer(
        &mut self,
        id_number: &str,
        password: &str,
    ) -> Result<bool, postgres::Error> {
        let rows = self.client.query(
            "SELECT id FROM organizers WHERE identification_number = $1 AND password_hash = $2",
            &[&id_number, &password],
        )?;
        Ok(!rows.is_empty())
    }

    /// Check whether a voter's certificate has been revoked.
    pub fn is_voter_revoked(&mut self, voter_id: i32) -> Result<bool, postgres::Error> {
        let rows = self.client.query(
            "SELECT certificate_revoked FROM voters WHERE id = $1",
            &[&voter_id],
        )?;
        Ok(rows.first().map(|r| r.get::<_, bool>(0)).unwrap_or(true))
    }

    /// Check whether an organizer's certificate has been revoked.
    pub fn is_organizer_revoked(&mut self, organizer_id: i32) -> Result<bool, postgres::Error> {
        let rows = self.client.query(
            "SELECT certificate_revoked FROM organizers WHERE id = $1",
            &[&organizer_id],
        )?;
        Ok(rows.first().map(|r| r.get::<_, bool>(0)).unwrap_or(true))
    }

    /// Increment failed login attempts for a voter. Returns the new count.
    pub fn increment_voter_failed_attempts(
        &mut self,
        voter_id: i32,
    ) -> Result<i32, postgres::Error> {
        let rows = self.client.query(
            "UPDATE voters SET failed_login_attempts = failed_login_attempts + 1 WHERE id = $1 RETURNING failed_login_attempts",
            &[&voter_id],
        )?;
        Ok(rows.first().map(|r| r.get::<_, i32>(0)).unwrap_or(0))
    }

    /// Increment failed login attempts for an organizer. Returns the new count.
    pub fn increment_organizer_failed_attempts(
        &mut self,
        organizer_id: i32,
    ) -> Result<i32, postgres::Error> {
        let rows = self.client.query(
            "UPDATE organizers SET failed_login_attempts = failed_login_attempts + 1 WHERE id = $1 RETURNING failed_login_attempts",
            &[&organizer_id],
        )?;
        Ok(rows.first().map(|r| r.get::<_, i32>(0)).unwrap_or(0))
    }

    /// Reset failed login attempts for a voter back to zero.
    pub fn reset_voter_failed_attempts(&mut self, voter_id: i32) -> Result<(), postgres::Error> {
        self.client.execute(
            "UPDATE voters SET failed_login_attempts = 0 WHERE id = $1",
            &[&voter_id],
        )?;
        Ok(())
    }

    /// Reset failed login attempts for an organizer back to zero.
    pub fn reset_organizer_failed_attempts(
        &mut self,
        organizer_id: i32,
    ) -> Result<(), postgres::Error> {
        self.client.execute(
            "UPDATE organizers SET failed_login_attempts = 0 WHERE id = $1",
            &[&organizer_id],
        )?;
        Ok(())
    }

    /// Revoke a voter's certificate (marks it in the user row).
    pub fn revoke_voter_certificate(&mut self, voter_id: i32) -> Result<(), postgres::Error> {
        self.client.execute(
            "UPDATE voters SET certificate_revoked = true WHERE id = $1",
            &[&voter_id],
        )?;
        Ok(())
    }

    /// Revoke an organizer's certificate (marks it in the user row).
    pub fn revoke_organizer_certificate(
        &mut self,
        organizer_id: i32,
    ) -> Result<(), postgres::Error> {
        self.client.execute(
            "UPDATE organizers SET certificate_revoked = true WHERE id = $1",
            &[&organizer_id],
        )?;
        Ok(())
    }
}
