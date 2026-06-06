use std::sync::mpsc;

use super::{Action, Scene};
use crate::event;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

#[derive(Default, PartialEq)]
enum AccountType {
    #[default]
    Organizer,
    Voter,
}

#[derive(Default, PartialEq)]
enum FocusField {
    #[default]
    Organization,
    FirstName,
    LastName,
    Username,
    Password,
}

pub struct RegisterScene {
    account_type: AccountType,
    focus: FocusField,
    organization: String,
    first_name: String,
    last_name: String,
    username: String,
    password: String,
}

impl RegisterScene {
    pub fn new() -> Self {
        RegisterScene {
            account_type: AccountType::default(),
            focus: FocusField::default(),
            organization: String::new(),
            first_name: String::new(),
            last_name: String::new(),
            username: String::new(),
            password: String::new(),
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
            .title(" Register ")
            .title_alignment(Alignment::Center)
            .border_set(border::PLAIN);

        frame.render_widget(&block, centered);

        let content_area = block.inner(centered);
        let [hdr_area, hr_area, input_area, ftr_area] = Layout::vertical([
            Constraint::Percentage(20),
            Constraint::Length(1),
            Constraint::Percentage(70),
            Constraint::Length(5),
        ])
        .areas(content_area);

        self.draw_header(frame, hdr_area);
        self.draw_hrule(frame, hr_area);
        self.draw_input_widget(frame, input_area);
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
                Line::from("Create an account").centered().bold(),
                Line::from(""),
                Line::from(vec![
                    Span::raw("I am a: "),
                    org_label,
                    voter_label,
                    Span::raw(" (Ctrl+t to toggle)").dark_gray(),
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
        let make_block = || Block::new().borders(Borders::ALL).border_style(border_style);

        match self.account_type {
            AccountType::Organizer => {
                let areas = Layout::vertical([
                    Constraint::Length(3),
                    Constraint::Length(3),
                ])
                .split(area);
                let org_focused = self.focus == FocusField::Organization;
                let pw_focused = self.focus == FocusField::Password;

                frame.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled(
                            if org_focused { "> " } else { "  " },
                            caret_color,
                        ),
                        Span::styled("Organization: ", Color::DarkGray),
                        Span::raw(&self.organization),
                    ]))
                    .block(make_block()),
                    areas[0],
                );
                frame.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled(
                            if pw_focused { "> " } else { "  " },
                            caret_color,
                        ),
                        Span::styled("Password: ", Color::DarkGray),
                        Span::raw("*".repeat(self.password.len())),
                    ]))
                    .block(make_block()),
                    areas[1],
                );
            }
            AccountType::Voter => {
                let areas = Layout::vertical([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Length(3),
                ])
                .split(area);
                let fn_focused = self.focus == FocusField::FirstName;
                let ln_focused = self.focus == FocusField::LastName;
                let un_focused = self.focus == FocusField::Username;
                let pw_focused = self.focus == FocusField::Password;

                frame.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled(
                            if fn_focused { "> " } else { "  " },
                            caret_color,
                        ),
                        Span::styled("First Name: ", Color::DarkGray),
                        Span::raw(&self.first_name),
                    ]))
                    .block(make_block()),
                    areas[0],
                );
                frame.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled(
                            if ln_focused { "> " } else { "  " },
                            caret_color,
                        ),
                        Span::styled("Last Name: ", Color::DarkGray),
                        Span::raw(&self.last_name),
                    ]))
                    .block(make_block()),
                    areas[1],
                );
                frame.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled(
                            if un_focused { "> " } else { "  " },
                            caret_color,
                        ),
                        Span::styled("Username: ", Color::DarkGray),
                        Span::raw(&self.username),
                    ]))
                    .block(make_block()),
                    areas[2],
                );
                frame.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled(
                            if pw_focused { "> " } else { "  " },
                            caret_color,
                        ),
                        Span::styled("Password: ", Color::DarkGray),
                        Span::raw("*".repeat(self.password.len())),
                    ]))
                    .block(make_block()),
                    areas[3],
                );
            }
        }
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Paragraph::new(Line::from("footer").centered().dark_gray()),
            area,
        );
    }

    pub fn handle(&mut self, key: KeyEvent) -> Action {
        if key.kind != KeyEventKind::Press {
            return Action::None;
        }
        match key.code {
            KeyCode::Char('t') if key.modifiers == KeyModifiers::CONTROL => {
                self.account_type = match self.account_type {
                    AccountType::Organizer => AccountType::Voter,
                    AccountType::Voter => AccountType::Organizer,
                };
                self.focus = match self.account_type {
                    AccountType::Organizer => FocusField::Organization,
                    AccountType::Voter => FocusField::FirstName,
                };
                Action::None
            }
            KeyCode::Char('l') if key.modifiers == KeyModifiers::CONTROL => {
                Action::SwitchScene(Scene::Login(super::login::LoginScene::new()))
            }
            KeyCode::Tab => {
                self.focus = next_focus(&self.focus, &self.account_type);
                Action::None
            }
            KeyCode::Backspace => {
                match self.focus {
                    FocusField::Organization => self.organization.pop(),
                    FocusField::FirstName => self.first_name.pop(),
                    FocusField::LastName => self.last_name.pop(),
                    FocusField::Username => self.username.pop(),
                    FocusField::Password => self.password.pop(),
                };
                Action::None
            }
            KeyCode::Char(c) => {
                match self.focus {
                    FocusField::Organization => self.organization.push(c),
                    FocusField::FirstName => self.first_name.push(c),
                    FocusField::LastName => self.last_name.push(c),
                    FocusField::Username => self.username.push(c),
                    FocusField::Password => self.password.push(c),
                };
                Action::None
            }
            _ => Action::None,
        }
    }

    pub fn on_enter(&mut self, _tx: mpsc::Sender<event::Event>) {}

    pub fn on_exit(&mut self) {}
}

fn next_focus(current: &FocusField, account_type: &AccountType) -> FocusField {
    match account_type {
        AccountType::Organizer => match current {
            FocusField::Organization => FocusField::Password,
            FocusField::Password => FocusField::Organization,
            _ => FocusField::Organization,
        },
        AccountType::Voter => match current {
            FocusField::FirstName => FocusField::LastName,
            FocusField::LastName => FocusField::Username,
            FocusField::Username => FocusField::Password,
            FocusField::Password => FocusField::FirstName,
            _ => FocusField::FirstName,
        },
    }
}
