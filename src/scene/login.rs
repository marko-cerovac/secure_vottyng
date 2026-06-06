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
use crate::event;

#[derive(Default)]
enum FocusField {
    #[default]
    Username,
    Password,
}

pub struct LoginScene {
    username: String,
    password: String,
    focus: FocusField,
    error: Option<String>,
}

impl LoginScene {
    pub fn new() -> Self {
        LoginScene {
            username: String::with_capacity(20),
            password: String::with_capacity(20),
            focus: FocusField::default(),
            error: None,
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let outer = Layout::vertical([
            Constraint::Percentage(15), // top padding
            Constraint::Percentage(70), // content area
            Constraint::Percentage(15), // bottom padding
        ]);
        let [_, vert_center, _] = outer.areas(area);

        let inner = Layout::horizontal([
            Constraint::Percentage(30), // left padding
            Constraint::Percentage(40), // content arrea
            Constraint::Percentage(30), // right padding
        ]);
        let [_, centered, _] = inner.areas(vert_center);

        let block = Block::bordered()
            .title(" Vottyng - Login ")
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
        let [_, middle, _, username_space, _, password_space, error_space, _] =
            layout.areas(content_area);

        let welcome_message = Paragraph::new(vec![
            Line::from(vec![
                Span::raw("Welcome to Secure Vo"),
                Span::styled("tty", Color::Red),
                Span::raw("ing"),
            ]).bold().centered(),
            Line::from(""),
            Line::from(vec![
                Span::raw("Press "),
                Span::raw(" Enter ").bold().black().on_red(),
                Span::raw(" to log in"),
            ]).centered().dark_gray(),
            Line::from(vec![
                Span::raw(" Ctrl+R ").bold().black().on_red(),
                Span::raw(" register | "),
                Span::raw(" Ctrl+C ").bold().black().on_red(),
                Span::raw(" exit"),
            ]).centered().dark_gray(),
        ]);

        let username_prefix = match self.focus {
            FocusField::Username => "> ",
            FocusField::Password => "  ",
        };
        let password_prefix = match self.focus {
            FocusField::Username => "  ",
            FocusField::Password => "> ",
        };

        let username_field = Paragraph::new(Line::from(vec![
            Span::styled(username_prefix, Color::Red),
            Span::styled("Username: ", Color::DarkGray),
            Span::styled(self.username.to_string(), Color::White),
        ]))
        .block(Block::new().borders(Borders::ALL))
        .dark_gray();

        let password_field = Paragraph::new(Line::from(vec![
            Span::styled(password_prefix, Color::Red),
            Span::styled("Password: ", Color::DarkGray),
            Span::styled("*".repeat(self.password.len()), Color::White),
        ]))
        .block(Block::new().borders(Borders::ALL))
        .dark_gray();

        frame.render_widget(&welcome_message, middle);
        frame.render_widget(&username_field, username_space);
        frame.render_widget(&password_field, password_space);

        if let Some(msg) = &self.error {
            let error_msg = Paragraph::new(Line::from(msg.as_str()).centered())
                .fg(Color::Red);
            frame.render_widget(&error_msg, error_space);
        }
    }

    pub fn handle(&mut self, key: KeyEvent) -> Action {
        if key.kind != KeyEventKind::Press {
            return Action::None;
        }
        match key.code {
            KeyCode::Enter => {
                if self.username.is_empty() || self.password.is_empty() {
                    self.error = Some("Username and password are required".into());
                    Action::None
                } else {
                    self.error = None;
                    Action::SwitchScene(Scene::Dashboard(
                        super::dashboard::DashboardScene::new(),
                    ))
                }
            }
            KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
                Action::SwitchScene(Scene::Register(super::register::RegisterScene::new()))
            }
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => Action::Quit,
            KeyCode::Tab => {
                self.focus = match self.focus {
                    FocusField::Username => FocusField::Password,
                    FocusField::Password => FocusField::Username,
                };
                self.error = None;
                Action::None
            }
            KeyCode::Backspace => {
                let s = match self.focus {
                    FocusField::Username => &mut self.username,
                    FocusField::Password => &mut self.password,
                };
                s.pop();
                self.error = None;
                Action::None
            }
            KeyCode::Char(c) => {
                let s = match self.focus {
                    FocusField::Username => &mut self.username,
                    FocusField::Password => &mut self.password,
                };
                s.push(c);
                self.error = None;
                Action::None
            }
            _ => Action::None,
        }
    }

    pub fn on_enter(&mut self, _tx: mpsc::Sender<event::Event>) {}
    pub fn on_exit(&mut self) {}
}
