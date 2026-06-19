use crate::event::Event;
use std::sync::mpsc;

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

impl DbResponse {
    pub fn send(self, ui_tx: &mpsc::Sender<Event>) -> bool {
        ui_tx.send(Event::DbResponse(self)).is_ok()
    }
}
