use postgres::{Client, NoTls};
use std::sync::mpsc;
use std::thread;

use crate::event::Event;
use crate::models::AccountRegistrationForm;
use crate::services::auth::AuthService;
use crate::services::user::UserService;

pub enum DbRequest {
    /// Step 1: validate a certificate PEM against the DB.
    ValidateCertificate {
        cert_pem: String,
        is_organizer: bool,
    },
    /// Step 2: authenticate with credentials (user was identified by cert in step 1).
    AuthenticateUser {
        user_id: i32,
        identifier: String,
        password: String,
        is_organizer: bool,
    },
    RegisterUser {
        form: AccountRegistrationForm,
    },
}

pub enum DbResponse {
    /// Certificate is valid; carries the user_id for step 2.
    CertificateValid {
        user_id: i32,
    },
    CertificateInvalid(String),
    AuthSuccess {
        is_organizer: bool,
    },
    AuthFailed(String),
    RegistrationOk,
    RegistrationFailed(String),
}

fn send_response(ui_tx: &mpsc::Sender<Event>, response: DbResponse) -> bool {
    ui_tx.send(Event::DbResponse(response)).is_ok()
}

pub fn spawn_db_worker(ui_tx: mpsc::Sender<Event>) -> mpsc::Sender<DbRequest> {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://secure_vottyng_user:kriptografija@localhost/secure_vottyng".into()
    });

    let (req_tx, req_rx) = mpsc::channel::<DbRequest>();

    thread::spawn(move || {
        let mut client = Client::connect(&url, NoTls).expect("failed to connect to database");

        let _ca_hierarchy = crate::crypto::ca::init_ca_hierarchy(&mut client);

        for request in req_rx {
            let result = match request {
                DbRequest::ValidateCertificate {
                    cert_pem,
                    is_organizer,
                } => {
                    let mut auth = AuthService::new(&mut client);
                    // TODO: add full cryptographic validation (time, issuer chain, CRL)
                    let found = if is_organizer {
                        auth.find_organizer_by_cert(&cert_pem)
                    } else {
                        auth.find_voter_by_cert(&cert_pem)
                    };
                    match found {
                        Ok(Some(user_id)) => DbResponse::CertificateValid { user_id },
                        Ok(None) => DbResponse::CertificateInvalid(
                            "Certificate not found or revoked".into(),
                        ),
                        Err(e) => DbResponse::CertificateInvalid(e.to_string()),
                    }
                }
                DbRequest::AuthenticateUser {
                    user_id,
                    identifier,
                    password,
                    is_organizer,
                } => {
                    let mut auth = AuthService::new(&mut client);
                    let authenticated = if is_organizer {
                        auth.authenticate_organizer(&identifier, &password)
                    } else {
                        auth.authenticate_voter(&identifier, &password)
                    };
                    match authenticated {
                        Ok(true) => {
                            // Reset failed attempts on success
                            let _ = if is_organizer {
                                auth.reset_organizer_failed_attempts(user_id)
                            } else {
                                auth.reset_voter_failed_attempts(user_id)
                            };
                            DbResponse::AuthSuccess { is_organizer }
                        }
                        Ok(false) => {
                            // Increment failed attempts, revoke cert after 3
                            let count = if is_organizer {
                                auth.increment_organizer_failed_attempts(user_id)
                            } else {
                                auth.increment_voter_failed_attempts(user_id)
                            };
                            if let Ok(n) = count {
                                if n >= 3 {
                                    let _ = if is_organizer {
                                        auth.revoke_organizer_certificate(user_id)
                                    } else {
                                        auth.revoke_voter_certificate(user_id)
                                    };
                                    if !send_response(
                                        &ui_tx,
                                        DbResponse::AuthFailed(
                                            "Certificate revoked after 3 failed attempts".into(),
                                        ),
                                    ) {
                                        return;
                                    }
                                    continue;
                                }
                            }
                            DbResponse::AuthFailed("Invalid credentials".into())
                        }
                        Err(e) => DbResponse::AuthFailed(e.to_string()),
                    }
                }
                DbRequest::RegisterUser { form } => {
                    match UserService::new(&mut client).register(form) {
                        Ok(_) => DbResponse::RegistrationOk,
                        Err(e) => DbResponse::RegistrationFailed(e.to_string()),
                    }
                }
            };
            if !send_response(&ui_tx, result) {
                break;
            }
        }
    });

    req_tx
}
