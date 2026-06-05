use std::io;

use crossterm::event::*;
use ratatui::DefaultTerminal;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::border;
use ratatui::widgets::{Block, Gauge, Widget};
use ratatui::{Frame, text::Line};

pub struct App {
    pub exit: bool,
    pub progress_bar_color: Color,
}

impl App {
    pub fn new() -> Self {
        App {
            exit: false,
            progress_bar_color: Color::Green,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        // the main event loop
        while !self.exit {
            // draw one frame
            terminal.draw(|frame| self.draw(frame))?;
            if let Event::Key(key) = crossterm::event::read()? {
                self.handle_key_event(key)?
            }
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        // for smaller applications, it's common to
        // implement the Widget trait directly on the
        // App struct to keep things in one place
        frame.render_widget(self, frame.area());
    }

    fn handle_key_event(&mut self, key: KeyEvent) -> io::Result<()> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Char('q') => {
                    self.exit = true;
                }
                KeyCode::Char('c') => {
                    self.progress_bar_color = self.get_next_color();
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn get_next_color(&self) -> Color {
        match self.progress_bar_color {
            Color::Green => Color::Yellow,
            Color::Yellow => Color::Red,
            Color::Red => Color::Green,
            _ => Color::Green,
        }
    }
}

// we implement the Widget trait on a
// reference to the App struct
impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let vertical_layout =
            Layout::vertical([Constraint::Percentage(20), Constraint::Percentage(80)]);
        let [title_area, gauge_area] = vertical_layout.areas(area);
        // render the title at the top of the layout
        Line::from("Process overview")
            .bold()
            .render(title_area, buf);

        let instructions = Line::from(vec![
            " Change color ".into(),
            "<C>".blue().bold(),
            " Quit ".into(),
            "<Q>".blue().bold(),
        ])
        .centered();

        let border = Block::bordered()
            .title(" Secure voting happening ")
            .title_bottom(instructions)
            .border_set(border::THICK);

        let progress_bar = Gauge::default()
            .gauge_style(Style::default().fg(self.progress_bar_color))
            .block(border)
            .label("Voting in progress")
            .ratio(0.5);

        // the first param takes an area.
        // we can create a rectangle to use as the area.
        //
        progress_bar.render(
            Rect {
                x: gauge_area.left(),
                y: gauge_area.top(),
                width: gauge_area.width,
                height: 3,
            },
            buf,
        );
    }
}
