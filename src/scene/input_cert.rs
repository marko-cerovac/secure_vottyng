use std::sync::mpsc;

use super::{Action, Scene};
use crate::db::{DbRequest, DbResponse};
use crate::event;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::Frame;

use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

#[derive(Default, PartialEq, Clone, Copy)]
enum AccountType {
    #[default]
    Organizer,
    Voter,
}

pub struct InputCertScene {
    account_type: AccountType,
    certificate: String,
    error: Option<String>,
    waiting: bool,
    db_tx: Option<mpsc::Sender<DbRequest>>,
}

impl InputCertScene {
    pub fn new() -> Self {
        InputCertScene {
            account_type: AccountType::default(),
            certificate: String::new(),
            error: None,
            waiting: false,
            db_tx: None,
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
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ]);
        let [_, centered, _] = inner.areas(vert_center);

        let block = Block::bordered()
            .title(" Login ")
            .title_alignment(Alignment::Center)
            .border_set(border::PLAIN);

        frame.render_widget(&block, centered);

        let content_area = block.inner(centered);
        let [hdr_area, hr_area, input_area, error_area, ftr_area] = Layout::vertical([
            Constraint::Percentage(20),
            Constraint::Length(1),
            Constraint::Percentage(60),
            Constraint::Length(3),
            Constraint::Length(5),
        ])
        .areas(content_area);

        self.draw_header(frame, hdr_area);
        self.draw_hrule(frame, hr_area);
        self.draw_input_widget(frame, input_area);
        self.draw_error(frame, error_area);
        self.draw_footer(frame, ftr_area);
    }

    fn draw_header(&self, frame: &mut Frame, area: Rect) {
        let (org_label, voter_label) = match self.account_type {
            AccountType::Organizer => (
                Span::raw(" Organizer ").black().on_red(),
                Span::raw(" Voter "),
            ),
            AccountType::Voter => (
                Span::raw(" Organizer "),
                Span::raw(" Voter ").black().on_blue(),
            ),
        };

        frame.render_widget(
            Paragraph::new(vec![
                Line::from(""),
                Line::from("Enter your certificate").centered().bold(),
                Line::from(""),
                Line::from(vec![
                    Span::raw("I am a: "),
                    org_label,
                    voter_label,
                    Span::raw(" (Ctrl+t)").dark_gray(),
                ])
                .centered(),
                Line::from(""),
            ]),
            area,
        );
    }

    fn draw_hrule(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Paragraph::new(
                Line::from("─".repeat(area.width as usize - 4))
                    .centered()
                    .dark_gray(),
            ),
            area,
        );
    }

    fn draw_input_widget(&self, frame: &mut Frame, area: Rect) {
        let caret_color = match self.account_type {
            AccountType::Organizer => Color::Red,
            AccountType::Voter => Color::Blue,
        };
        let border_style = Style::new().fg(Color::DarkGray);

        let label = if self.waiting {
            "Validating..."
        } else {
            "Certificate: "
        };

        let input = Paragraph::new(Line::from(vec![
            Span::styled("> ", caret_color),
            Span::styled(label, Color::DarkGray),
            Span::raw(&self.certificate),
        ]))
        .block(
            Block::new()
                .borders(Borders::ALL)
                .border_style(border_style),
        );

        frame.render_widget(input, area);
    }

    fn draw_error(&self, frame: &mut Frame, area: Rect) {
        if let Some(msg) = &self.error {
            let error_msg = Paragraph::new(Line::from(msg.as_str()).centered()).fg(Color::Red);
            frame.render_widget(error_msg, area);
        }
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::raw("Ctrl+"),
                    Span::raw("T").bold(),
                    Span::raw(" toggle  |  "),
                    Span::raw("Enter").bold(),
                    Span::raw(" submit  |  "),
                    Span::raw("Ctrl+"),
                    Span::raw("R").bold(),
                    Span::raw(" register  |  "),
                    Span::raw("Ctrl+"),
                    Span::raw("C").bold(),
                    Span::raw(" exit"),
                ])
                .centered()
                .dark_gray(),
            ]),
            area,
        );
    }

    pub fn handle(&mut self, key: KeyEvent) -> Action {
        if key.kind != KeyEventKind::Press {
            return Action::None;
        }
        if self.waiting {
            return Action::None;
        }
        match key.code {
            KeyCode::Char('t') if key.modifiers == KeyModifiers::CONTROL => {
                self.account_type = match self.account_type {
                    AccountType::Organizer => AccountType::Voter,
                    AccountType::Voter => AccountType::Organizer,
                };
                self.error = None;
                Action::None
            }
            KeyCode::Char('r') if key.modifiers == KeyModifiers::CONTROL => {
                Action::SwitchScene(Scene::Register(super::register::RegisterScene::new()))
            }
            KeyCode::Enter => {
                if self.certificate.is_empty() {
                    self.error = Some("Please enter your certificate".into());
                    Action::None
                } else if let Some(db_tx) = &self.db_tx {
                    let is_organizer = self.account_type == AccountType::Organizer;
                    let _ = db_tx.send(DbRequest::ValidateCertificate {
                        cert_pem: self.certificate.clone(),
                        is_organizer,
                    });
                    self.waiting = true;
                    self.error = None;
                    Action::None
                } else {
                    Action::None
                }
            }
            KeyCode::Backspace => {
                self.certificate.pop();
                self.error = None;
                Action::None
            }
            KeyCode::Char(c) => {
                self.certificate.push(c);
                self.error = None;
                Action::None
            }
            _ => Action::None,
        }
    }

    pub fn handle_paste(&mut self, text: &str) -> Action {
        if !self.waiting {
            self.certificate.push_str(text);
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
            DbResponse::CertificateValid { user_id } => {
                let is_organizer = self.account_type == AccountType::Organizer;
                Action::SwitchScene(Scene::Login(super::login::LoginScene::new_with_cert(
                    user_id,
                    is_organizer,
                )))
            }
            DbResponse::CertificateInvalid(reason) => {
                self.error = Some(reason);
                Action::None
            }
            _ => Action::None,
        }
    }
}
