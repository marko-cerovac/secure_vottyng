use ratatui::style::{Color};

pub struct LoginView {
    progress_bar_color: Color,
}

impl LoginView {
    pub fn new() -> Self {
        LoginView {
            progress_bar_color: Color::Green,
        }
    }
}
