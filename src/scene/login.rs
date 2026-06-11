use std::sync::mpsc;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::Color;
use ratatui::style::Stylize;
use ratatui::symbols::border;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use super::{Action, Scene};
use crate::db::{DbRequest, DbResponse};
use crate::event;

#[derive(Default)]
enum FocusField {
    #[default]
    Identifier,
    Password,
}

pub struct LoginScene {
    /// User ID from certificate validation (step 1).
    user_id: i32,
    is_organizer: bool,
    identifier: String,
    password: String,
    focus: FocusField,
    error: Option<String>,
    waiting: bool,
    db_tx: Option<mpsc::Sender<DbRequest>>,
}

impl LoginScene {
    /// Legacy constructor (used by dashboard's "back to login" — goes to InputCert instead).
    pub fn new() -> Self {
        LoginScene {
            user_id: 0,
            is_organizer: false,
            identifier: String::with_capacity(20),
            password: String::with_capacity(20),
            focus: FocusField::default(),
            error: None,
            waiting: false,
            db_tx: None,
        }
    }

    /// Construct from step 1 (certificate validated).
    pub fn new_with_cert(user_id: i32, is_organizer: bool) -> Self {
        LoginScene {
            user_id,
            is_organizer,
            identifier: String::with_capacity(20),
            password: String::with_capacity(20),
            focus: FocusField::default(),
            error: None,
            waiting: false,
            db_tx: None,
        }
    }

    fn identifier_label(&self) -> &'static str {
        if self.is_organizer {
            "ID Number: "
        } else {
            "Username: "
        }
    }

    fn accent_color(&self) -> Color {
        if self.is_organizer {
            Color::Red
        } else {
            Color::Blue
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let outer = Layout::vertical([
            Constraint::Percentage(15),
            Constraint::Percentage(70),
            Constraint::Percentage(15),
        ]);
        let [_, vert_center, _] = outer.areas(area);

        let inner = Layout::horizontal([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ]);
        let [_, centered, _] = inner.areas(vert_center);

        let account_label = if self.is_organizer {
            "Organizer"
        } else {
            "Voter"
        };
        let block = Block::bordered()
            .title(format!(" Login - {} ", account_label))
            .title_alignment(Alignment::Center)
            .border_set(border::PLAIN);

        frame.render_widget(&block, centered);

        let content_area = block.inner(centered);
        let layout = Layout::vertical([
            Constraint::Percentage(15),
            Constraint::Length(5),
            Constraint::Percentage(8),
            Constraint::Length(3),
            Constraint::Percentage(8),
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Percentage(20),
        ]);
        let [_, middle, _, id_space, _, password_space, error_space, _] =
            layout.areas(content_area);

        let accent = self.accent_color();

        let welcome_message = Paragraph::new(vec![
            Line::from(vec![
                Span::raw("Welcome to Secure Vo"),
                Span::styled("tty", accent),
                Span::raw("ing"),
            ])
            .bold()
            .centered(),
            Line::from(""),
            Line::from(vec![
                Span::raw("Press "),
                Span::raw(" Enter ").bold(),
                Span::raw(" to log in"),
            ])
            .centered()
            .dark_gray(),
            Line::from(vec![
                Span::raw(" Ctrl+B ").bold(),
                Span::raw(" back | "),
                Span::raw(" Ctrl+C ").bold(),
                Span::raw(" exit"),
            ])
            .centered()
            .dark_gray(),
        ]);

        let id_prefix = match self.focus {
            FocusField::Identifier => "> ",
            FocusField::Password => "  ",
        };
        let pw_prefix = match self.focus {
            FocusField::Identifier => "  ",
            FocusField::Password => "> ",
        };

        let id_field = Paragraph::new(Line::from(vec![
            Span::styled(id_prefix, accent),
            Span::styled(self.identifier_label(), Color::DarkGray),
            Span::styled(self.identifier.to_string(), Color::White),
        ]))
        .block(Block::new().borders(Borders::ALL))
        .dark_gray();

        let password_field = Paragraph::new(Line::from(vec![
            Span::styled(pw_prefix, accent),
            Span::styled("Password: ", Color::DarkGray),
            Span::styled("*".repeat(self.password.len()), Color::White),
        ]))
        .block(Block::new().borders(Borders::ALL))
        .dark_gray();

        frame.render_widget(&welcome_message, middle);
        frame.render_widget(&id_field, id_space);
        frame.render_widget(&password_field, password_space);

        if let Some(msg) = &self.error {
            let error_msg = Paragraph::new(Line::from(msg.as_str()).centered()).fg(Color::Red);
            frame.render_widget(&error_msg, error_space);
        }
    }

    pub fn handle(&mut self, key: KeyEvent) -> Action {
        if key.kind != KeyEventKind::Press {
            return Action::None;
        }
        if self.waiting {
            return Action::None;
        }
        match key.code {
            KeyCode::Enter => {
                if self.identifier.is_empty() || self.password.is_empty() {
                    self.error = Some("All fields are required".into());
                    Action::None
                } else if let Some(db_tx) = &self.db_tx {
                    let _ = db_tx.send(DbRequest::AuthenticateUser {
                        user_id: self.user_id,
                        identifier: self.identifier.clone(),
                        password: self.password.clone(),
                        is_organizer: self.is_organizer,
                    });
                    self.waiting = true;
                    self.error = None;
                    Action::None
                } else {
                    Action::None
                }
            }
            KeyCode::Char('b') if key.modifiers == KeyModifiers::CONTROL => {
                Action::SwitchScene(Scene::InputCert(super::input_cert::InputCertScene::new()))
            }
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => Action::Quit,
            KeyCode::Tab => {
                self.focus = match self.focus {
                    FocusField::Identifier => FocusField::Password,
                    FocusField::Password => FocusField::Identifier,
                };
                self.error = None;
                Action::None
            }
            KeyCode::Backspace => {
                let s = match self.focus {
                    FocusField::Identifier => &mut self.identifier,
                    FocusField::Password => &mut self.password,
                };
                s.pop();
                self.error = None;
                Action::None
            }
            KeyCode::Char(c) => {
                let s = match self.focus {
                    FocusField::Identifier => &mut self.identifier,
                    FocusField::Password => &mut self.password,
                };
                s.push(c);
                self.error = None;
                Action::None
            }
            _ => Action::None,
        }
    }

    pub fn handle_paste(&mut self, text: &str) -> Action {
        if !self.waiting {
            match self.focus {
                FocusField::Identifier => self.identifier.push_str(text),
                FocusField::Password => self.password.push_str(text),
            }
            self.error = None;
        }
        Action::None
    }

    pub fn on_enter(&mut self, _tx: mpsc::Sender<event::Event>, db: mpsc::Sender<DbRequest>) {
        self.db_tx = Some(db);
    }

    pub fn on_exit(&mut self) {}

    pub fn on_db_response(&mut self, response: DbResponse) -> Action {
        self.waiting = false;
        match response {
            DbResponse::AuthSuccess { is_organizer: _ } => {
                Action::SwitchScene(Scene::Dashboard(super::dashboard::DashboardScene::new()))
            }
            DbResponse::AuthFailed(reason) => {
                self.password.clear();
                self.error = Some(reason);
                Action::None
            }
            _ => Action::None,
        }
    }
}
