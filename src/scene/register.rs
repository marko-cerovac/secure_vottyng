use std::sync::mpsc;

use super::{Action, Scene};
use crate::db::DbRequest;
use crate::event;
use crate::models::AccountRegistrationForm;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::Frame;

use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

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
    account_form: AccountRegistrationForm,
    focus: FocusField,
    db_tx: Option<mpsc::Sender<DbRequest>>,
}

impl RegisterScene {
    pub fn new() -> Self {
        RegisterScene {
            account_form: AccountRegistrationForm::default(),
            focus: FocusField::default(),
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
        let (org_label, voter_label) = match &self.account_form {
            AccountRegistrationForm::Organizer { .. } => (
                Span::raw(" Organizer ").black().on_red(),
                Span::raw(" Voter "),
            ),
            AccountRegistrationForm::User { .. } => (
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
        let caret_color = match &self.account_form {
            AccountRegistrationForm::Organizer { .. } => Color::Red,
            AccountRegistrationForm::User { .. } => Color::Blue,
        };
        let border_style = Style::new().fg(Color::DarkGray);
        let make_block = || {
            Block::new()
                .borders(Borders::ALL)
                .border_style(border_style)
        };

        match &self.account_form {
            AccountRegistrationForm::Organizer {
                organization,
                password,
            } => {
                let areas =
                    Layout::vertical([Constraint::Length(3), Constraint::Length(3)]).split(area);
                let org_focused = self.focus == FocusField::Organization;
                let pw_focused = self.focus == FocusField::Password;

                frame.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled(if org_focused { "> " } else { "  " }, caret_color),
                        Span::styled("Organization: ", Color::DarkGray),
                        Span::raw(organization),
                    ]))
                    .block(make_block()),
                    areas[0],
                );
                frame.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled(if pw_focused { "> " } else { "  " }, caret_color),
                        Span::styled("Password: ", Color::DarkGray),
                        Span::raw("*".repeat(password.len())),
                    ]))
                    .block(make_block()),
                    areas[1],
                );
            }
            AccountRegistrationForm::User {
                f_name,
                l_name,
                username,
                password,
            } => {
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
                        Span::styled(if fn_focused { "> " } else { "  " }, caret_color),
                        Span::styled("First Name: ", Color::DarkGray),
                        Span::raw(f_name),
                    ]))
                    .block(make_block()),
                    areas[0],
                );
                frame.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled(if ln_focused { "> " } else { "  " }, caret_color),
                        Span::styled("Last Name: ", Color::DarkGray),
                        Span::raw(l_name),
                    ]))
                    .block(make_block()),
                    areas[1],
                );
                frame.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled(if un_focused { "> " } else { "  " }, caret_color),
                        Span::styled("Username: ", Color::DarkGray),
                        Span::raw(username),
                    ]))
                    .block(make_block()),
                    areas[2],
                );
                frame.render_widget(
                    Paragraph::new(Line::from(vec![
                        Span::styled(if pw_focused { "> " } else { "  " }, caret_color),
                        Span::styled("Password: ", Color::DarkGray),
                        Span::raw("*".repeat(password.len())),
                    ]))
                    .block(make_block()),
                    areas[3],
                );
            }
        }
    }

    fn draw_footer(&self, frame: &mut Frame, area: Rect) {
        frame.render_widget(
            Paragraph::new(vec![
                Line::from(vec![
                    Span::raw("Ctrl+"),
                    Span::raw("T").bold(),
                    Span::raw(" toggle  |  "),
                    Span::raw("Tab").bold(),
                    Span::raw(" focus next  |  "),
                    Span::raw("Enter").bold(),
                    Span::raw(" register  |  "),
                    Span::raw("Ctrl+"),
                    Span::raw("L").bold(),
                    Span::raw(" login  |  "),
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
        match key.code {
            KeyCode::Char('t') if key.modifiers == KeyModifiers::CONTROL => {
                self.account_form = match self.account_form {
                    AccountRegistrationForm::Organizer { .. } => AccountRegistrationForm::User {
                        f_name: String::new(),
                        l_name: String::new(),
                        username: String::new(),
                        password: String::new(),
                    },
                    AccountRegistrationForm::User { .. } => AccountRegistrationForm::Organizer {
                        organization: String::new(),
                        password: String::new(),
                    },
                };
                self.focus = match self.account_form {
                    AccountRegistrationForm::Organizer { .. } => FocusField::Organization,
                    AccountRegistrationForm::User { .. } => FocusField::FirstName,
                };
                Action::None
            }
            KeyCode::Char('l') if key.modifiers == KeyModifiers::CONTROL => {
                Action::SwitchScene(Scene::Login(super::login::LoginScene::new()))
            }
            KeyCode::Enter => {
                // Send registration request to database
                if let Some(db_tx) = &self.db_tx {
                    let _ = db_tx.send(DbRequest::RegisterUser {
                        form: self.account_form.clone(),
                    });
                }
                Action::None
            }
            KeyCode::Tab => {
                self.focus = next_focus(&self.focus, &self.account_form);
                Action::None
            }
            KeyCode::Backspace => {
                match (&mut self.account_form, &self.focus) {
                    (
                        AccountRegistrationForm::Organizer {
                            organization,
                            password: _,
                        },
                        FocusField::Organization,
                    ) => {
                        organization.pop();
                    }
                    (
                        AccountRegistrationForm::Organizer {
                            organization: _,
                            password,
                        },
                        FocusField::Password,
                    ) => {
                        password.pop();
                    }
                    (
                        AccountRegistrationForm::User {
                            f_name,
                            l_name: _,
                            username: _,
                            password: _,
                        },
                        FocusField::FirstName,
                    ) => {
                        f_name.pop();
                    }
                    (
                        AccountRegistrationForm::User {
                            f_name: _,
                            l_name,
                            username: _,
                            password: _,
                        },
                        FocusField::LastName,
                    ) => {
                        l_name.pop();
                    }
                    (
                        AccountRegistrationForm::User {
                            f_name: _,
                            l_name: _,
                            username,
                            password: _,
                        },
                        FocusField::Username,
                    ) => {
                        username.pop();
                    }
                    (
                        AccountRegistrationForm::User {
                            f_name: _,
                            l_name: _,
                            username: _,
                            password,
                        },
                        FocusField::Password,
                    ) => {
                        password.pop();
                    }
                    _ => {}
                }
                Action::None
            }
            KeyCode::Char(c) => {
                match (&mut self.account_form, &self.focus) {
                    (
                        AccountRegistrationForm::Organizer {
                            organization,
                            password,
                        },
                        FocusField::Organization,
                    ) => {
                        organization.push(c);
                    }
                    (
                        AccountRegistrationForm::Organizer {
                            organization,
                            password,
                        },
                        FocusField::Password,
                    ) => {
                        password.push(c);
                    }
                    (
                        AccountRegistrationForm::User {
                            f_name,
                            l_name,
                            username,
                            password,
                        },
                        FocusField::FirstName,
                    ) => {
                        f_name.push(c);
                    }
                    (
                        AccountRegistrationForm::User {
                            f_name,
                            l_name,
                            username,
                            password,
                        },
                        FocusField::LastName,
                    ) => {
                        l_name.push(c);
                    }
                    (
                        AccountRegistrationForm::User {
                            f_name,
                            l_name,
                            username,
                            password,
                        },
                        FocusField::Username,
                    ) => {
                        username.push(c);
                    }
                    (
                        AccountRegistrationForm::User {
                            f_name,
                            l_name,
                            username,
                            password,
                        },
                        FocusField::Password,
                    ) => {
                        password.push(c);
                    }
                    _ => {}
                }
                Action::None
            }
            _ => Action::None,
        }
    }

    pub fn handle_paste(&mut self, text: &str) -> Action {
        match (&mut self.account_form, &self.focus) {
            (
                AccountRegistrationForm::Organizer {
                    organization,
                    password,
                },
                FocusField::Organization,
            ) => {
                organization.push_str(text);
            }
            (
                AccountRegistrationForm::Organizer {
                    organization,
                    password,
                },
                FocusField::Password,
            ) => {
                password.push_str(text);
            }
            (
                AccountRegistrationForm::User {
                    f_name,
                    l_name,
                    username,
                    password,
                },
                FocusField::FirstName,
            ) => {
                f_name.push_str(text);
            }
            (
                AccountRegistrationForm::User {
                    f_name,
                    l_name,
                    username,
                    password,
                },
                FocusField::LastName,
            ) => {
                l_name.push_str(text);
            }
            (
                AccountRegistrationForm::User {
                    f_name,
                    l_name,
                    username,
                    password,
                },
                FocusField::Username,
            ) => {
                username.push_str(text);
            }
            (
                AccountRegistrationForm::User {
                    f_name,
                    l_name,
                    username,
                    password,
                },
                FocusField::Password,
            ) => {
                password.push_str(text);
            }
            _ => {}
        }
        Action::None
    }

    pub fn on_enter(
        &mut self,
        _tx: mpsc::Sender<event::Event>,
        db: std::sync::mpsc::Sender<DbRequest>,
    ) {
        self.db_tx = Some(db);
    }

    pub fn on_exit(&mut self) {}

    pub fn on_db_response(&mut self, _response: crate::db::DbResponse) -> Action {
        Action::None
    }
}

fn next_focus(current: &FocusField, account_form: &AccountRegistrationForm) -> FocusField {
    match account_form {
        AccountRegistrationForm::Organizer { .. } => match current {
            FocusField::Organization => FocusField::Password,
            FocusField::Password => FocusField::Organization,
            _ => FocusField::Organization,
        },
        AccountRegistrationForm::User { .. } => match current {
            FocusField::FirstName => FocusField::LastName,
            FocusField::LastName => FocusField::Username,
            FocusField::Username => FocusField::Password,
            FocusField::Password => FocusField::FirstName,
            _ => FocusField::FirstName,
        },
    }
}
