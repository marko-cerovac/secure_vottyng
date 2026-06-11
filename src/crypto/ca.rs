use rcgen::{
    BasicConstraints, CertificateParams, DnType, IsCa, Issuer, KeyPair, KeyUsagePurpose,
    date_time_ymd,
};

/// Holds the CA's data in memory.
/// Consists of two parts:
/// - cert_pem: the certificate in PEM format
/// - key_pair: the CA's private key
///
/// An [`Issuer`] for signing child certificates can be obtained via [`CaBundle::issuer`].
pub struct CaBundle {
    pub cert_pem: String,
    pub key_pair: KeyPair,
}

impl CaBundle {
    /// Build an [`Issuer`] that can sign child certificates.
    pub fn issuer(&self) -> Issuer<'_, &KeyPair> {
        Issuer::from_ca_cert_pem(&self.cert_pem, &self.key_pair)
            .expect("failed to parse CA cert PEM into Issuer")
    }
}

/// All three CAs needed by the system.
pub struct CaHierarchy {
    pub root: CaBundle,
    pub organizational: CaBundle,
    pub voter: CaBundle,
}

/// Initialise the CA hierarchy.
///
/// If all three rows already exist in `certificate_authorities`, load them.
/// Otherwise drop any partial state and generate fresh CAs.
pub fn init_ca_hierarchy(client: &mut postgres::Client) -> CaHierarchy {
    if let Some(hierarchy) = try_load_existing(client) {
        return hierarchy;
    }

    let hierarchy = generate_hierarchy();
    store_hierarchy(client, &hierarchy);
    hierarchy
}

// ---------------------------------------------------------------------------
// Loading existing CAs from the database
// ---------------------------------------------------------------------------

fn try_load_existing(client: &mut postgres::Client) -> Option<CaHierarchy> {
    let rows = client
        .query(
            "SELECT ca_type, certificate, private_key FROM certificate_authorities ORDER BY id",
            &[],
        )
        .ok()?;

    if rows.len() < 3 {
        return None;
    }

    let mut root: Option<CaBundle> = None;
    let mut org: Option<CaBundle> = None;
    let mut voter: Option<CaBundle> = None;

    for row in &rows {
        let ca_type: &str = row.get("ca_type");
        let cert_pem: &str = row.get("certificate");
        let key_pem: &str = row.get("private_key");

        let key_pair = KeyPair::from_pem(key_pem).ok()?;

        let bundle = CaBundle {
            cert_pem: cert_pem.to_owned(),
            key_pair,
        };

        match ca_type {
            "root" => root = Some(bundle),
            "organizational" => org = Some(bundle),
            "voter" => voter = Some(bundle),
            _ => {}
        }
    }

    Some(CaHierarchy {
        root: root?,
        organizational: org?,
        voter: voter?,
    })
}

// ---------------------------------------------------------------------------
// Generating a fresh CA hierarchy
// ---------------------------------------------------------------------------

fn generate_hierarchy() -> CaHierarchy {
    // ---- Root CA (self-signed) ----
    let root_key = KeyPair::generate().expect("failed to generate root CA key pair");

    let mut root_params = CertificateParams::default();
    root_params.is_ca = IsCa::Ca(BasicConstraints::Constrained(1));
    root_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    root_params.not_before = date_time_ymd(2025, 1, 1);
    root_params.not_after = date_time_ymd(2035, 1, 1);
    root_params
        .distinguished_name
        .push(DnType::CommonName, "Secure Vottyng Root CA");
    root_params
        .distinguished_name
        .push(DnType::OrganizationName, "Secure Vottyng System");

    let root_cert = root_params
        .self_signed(&root_key)
        .expect("failed to self-sign root CA");
    let root_cert_pem = root_cert.pem();

    // ---- Organizational CA (signed by Root) ----
    let org_key = KeyPair::generate().expect("failed to generate org CA key pair");

    let mut org_params = CertificateParams::default();
    org_params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
    org_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    org_params.not_before = date_time_ymd(2025, 1, 1);
    org_params.not_after = date_time_ymd(2035, 1, 1);
    org_params.use_authority_key_identifier_extension = true;
    org_params
        .distinguished_name
        .push(DnType::CommonName, "Organizational CA");
    org_params
        .distinguished_name
        .push(DnType::OrganizationName, "Secure Vottyng System");

    let root_issuer = Issuer::from_params(&root_params, &root_key);
    let org_cert = org_params
        .signed_by(&org_key, &root_issuer)
        .expect("failed to sign org CA cert");
    let org_cert_pem = org_cert.pem();

    // ---- Voter CA (signed by Root) ----
    let voter_key = KeyPair::generate().expect("failed to generate voter CA key pair");

    let mut voter_params = CertificateParams::default();
    voter_params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));
    voter_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];
    voter_params.not_before = date_time_ymd(2025, 1, 1);
    voter_params.not_after = date_time_ymd(2035, 1, 1);
    voter_params.use_authority_key_identifier_extension = true;
    voter_params
        .distinguished_name
        .push(DnType::CommonName, "Voter CA");
    voter_params
        .distinguished_name
        .push(DnType::OrganizationName, "Secure Voting System");

    let voter_cert = voter_params
        .signed_by(&voter_key, &root_issuer)
        .expect("failed to sign voter CA cert");
    let voter_cert_pem = voter_cert.pem();

    CaHierarchy {
        root: CaBundle {
            cert_pem: root_cert_pem,
            key_pair: root_key,
        },
        organizational: CaBundle {
            cert_pem: org_cert_pem,
            key_pair: org_key,
        },
        voter: CaBundle {
            cert_pem: voter_cert_pem,
            key_pair: voter_key,
        },
    }
}

// ---------------------------------------------------------------------------
// Persisting CAs to the database
// ---------------------------------------------------------------------------

fn store_hierarchy(client: &mut postgres::Client, h: &CaHierarchy) {
    // Clear any partial state from a prior failed run.
    client
        .execute("DELETE FROM certificate_authorities", &[])
        .expect("failed to clear certificate_authorities");

    let stmt = "INSERT INTO certificate_authorities (ca_type, certificate, private_key) \
                VALUES ($1, $2, $3)";

    for (ca_type, bundle) in [
        ("root", &h.root),
        ("organizational", &h.organizational),
        ("voter", &h.voter),
    ] {
        let key_pem = bundle.key_pair.serialize_pem();
        client
            .execute(stmt, &[&ca_type, &bundle.cert_pem, &key_pem])
            .expect("failed to insert CA into database");
    }
}
