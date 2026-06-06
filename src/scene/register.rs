use crate::event;
use super::{Action, Scene};
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::style::Color;
use ratatui::symbols::border;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph};
use ratatui::Frame;

pub struct RegisterScene;

impl RegisterScene {
    pub fn new() -> Self {
        RegisterScene
    }

    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();

        let outer = Layout::vertical([
            Constraint::Percentage(20), // top padding
            Constraint::Percentage(60), // content area
            Constraint::Percentage(20), // bottom padding
        ]);
        let [_, vert_center, _] = outer.areas(area);

        let inner = Layout::horizontal([
            Constraint::Percentage(30), // left padding
            Constraint::Percentage(40), // content arrea
            Constraint::Percentage(30), // right padding
        ]);
        let [_, centered, _] = inner.areas(vert_center);
    }

}
