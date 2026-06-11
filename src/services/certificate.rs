use postgres::Client;

use crate::models::{CaType, CertificateAuthority, CrlEntry};

pub struct CertificateService<'a> {
    client: &'a mut Client,
}

impl<'a> CertificateService<'a> {
    pub fn new(client: &'a mut Client) -> Self {
        CertificateService { client }
    }

    /// Fetch a CA by type (root, organizational, voter).
    pub fn get_ca(
        &mut self,
        ca_type: &CaType,
    ) -> Result<Option<CertificateAuthority>, postgres::Error> {
        let rows = self.client.query(
            "SELECT id, ca_type, certificate, private_key, created_at FROM certificate_authorities WHERE ca_type = $1",
            &[&ca_type.as_str()],
        )?;
        Ok(rows.first().map(|row| row_to_ca(row)))
    }

    /// Store a new CA. Returns the CA ID.
    pub fn store_ca(
        &mut self,
        ca_type: &CaType,
        certificate: &str,
        private_key: &str,
    ) -> Result<i32, postgres::Error> {
        let rows = self.client.query(
            "INSERT INTO certificate_authorities (ca_type, certificate, private_key) VALUES ($1, $2, $3) RETURNING id",
            &[&ca_type.as_str(), &certificate, &private_key],
        )?;
        Ok(rows[0].get(0))
    }

    /// Add a CRL entry (revoke a certificate by serial number).
    pub fn add_crl_entry(
        &mut self,
        ca_type: &CaType,
        serial_number: &str,
        reason: Option<&str>,
    ) -> Result<(), postgres::Error> {
        self.client.execute(
            "INSERT INTO crl_entries (ca_type, serial_number, reason) VALUES ($1, $2, $3) ON CONFLICT (ca_type, serial_number) DO NOTHING",
            &[&ca_type.as_str(), &serial_number, &reason],
        )?;
        Ok(())
    }

    /// Check whether a certificate serial number is on the CRL.
    pub fn is_revoked(
        &mut self,
        ca_type: &CaType,
        serial_number: &str,
    ) -> Result<bool, postgres::Error> {
        let rows = self.client.query(
            "SELECT id FROM crl_entries WHERE ca_type = $1 AND serial_number = $2",
            &[&ca_type.as_str(), &serial_number],
        )?;
        Ok(!rows.is_empty())
    }

    /// Get all CRL entries for a given CA type.
    pub fn get_crl(&mut self, ca_type: &CaType) -> Result<Vec<CrlEntry>, postgres::Error> {
        let rows = self.client.query(
            "SELECT id, ca_type, serial_number, revoked_at, reason FROM crl_entries WHERE ca_type = $1 ORDER BY revoked_at DESC",
            &[&ca_type.as_str()],
        )?;
        Ok(rows
            .iter()
            .map(|row| {
                let ca_type_str: String = row.get(1);
                CrlEntry {
                    id: row.get(0),
                    ca_type: str_to_ca_type(&ca_type_str),
                    serial_number: row.get(2),
                    revoked_at: row.get(3),
                    reason: row.get(4),
                }
            })
            .collect())
    }
}

fn row_to_ca(row: &postgres::Row) -> CertificateAuthority {
    let ca_type_str: String = row.get(1);
    CertificateAuthority {
        id: row.get(0),
        ca_type: str_to_ca_type(&ca_type_str),
        certificate: row.get(2),
        private_key: row.get(3),
        created_at: row.get(4),
    }
}

fn str_to_ca_type(s: &str) -> CaType {
    match s {
        "organizational" => CaType::Organizational,
        "voter" => CaType::Voter,
        _ => CaType::Root,
    }
}
