mod login;
mod dashboard;

pub use login::*;
pub use dashboard::*;

use ratatui::style::Color;

pub enum Scene {
    Dashboard,
    LoginView,
}
