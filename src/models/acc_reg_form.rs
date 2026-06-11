#[derive(Debug, Clone)]
pub enum AccountRegistrationForm {
    Organizer {
        organization: String,
        password: String,
    },
    User {
        f_name: String,
        l_name: String,
        username: String,
        password: String,
    },
}

impl Default for AccountRegistrationForm {
    fn default() -> Self {
        AccountRegistrationForm::Organizer {
            organization: String::new(),
            password: String::new(),
        }
    }
}
