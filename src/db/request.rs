use crate::models::AccountRegistrationForm;

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
    RegisterUserWithCertificate {
        form: AccountRegistrationForm,
    },
}
