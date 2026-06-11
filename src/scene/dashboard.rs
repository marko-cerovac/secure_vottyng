use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Gauge, Widget};

use super::Action;
use crate::db::DbRequest;
use crate::event;

pub struct DashboardScene {
    pub progress_bar_color: Color,
    pub progress: f64,
    stop_flag: Option<Arc<AtomicBool>>,
    reset_flag: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl DashboardScene {
    pub fn new() -> Self {
        DashboardScene {
            progress_bar_color: Color::Green,
            progress: 0.0,
            stop_flag: None,
            reset_flag: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        let vertical_layout =
            Layout::vertical([Constraint::Percentage(20), Constraint::Percentage(80)]);
        let [title_area, gauge_area] = vertical_layout.areas(area);

        Line::from("Process overview")
            .bold()
            .render(title_area, frame.buffer_mut());

        let instructions = Line::from(vec![
            " Change color ".into(),
            "<C>".blue().bold(),
            " Reset ".into(),
            "<R>".blue().bold(),
            " Back to login ".into(),
            "<L>".blue().bold(),
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
            .label(format!(
                "Voting in progress: {:.2}%",
                self.progress * 100_f64
            ))
            .ratio(self.progress);

        progress_bar.render(
            Rect {
                x: gauge_area.left(),
                y: gauge_area.top(),
                width: gauge_area.width,
                height: 3,
            },
            frame.buffer_mut(),
        );
    }

    pub fn handle(&mut self, key: KeyEvent) -> Action {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Char('c') => {
                    self.progress_bar_color = self.get_next_color();
                }
                KeyCode::Char('r') => {
                    self.reset_flag.store(true, Ordering::Relaxed);
                }
                KeyCode::Char('l') => {
                    return Action::SwitchScene(super::Scene::InputCert(
                        super::input_cert::InputCertScene::new(),
                    ));
                }
                _ => {}
            }
        }
        Action::None
    }

    pub fn on_enter(
        &mut self,
        tx: mpsc::Sender<event::Event>,
        _db: std::sync::mpsc::Sender<DbRequest>,
    ) {
        let stop = Arc::new(AtomicBool::new(false));
        let stop_clone = stop.clone();
        let reset = self.reset_flag.clone();
        let tx = tx.clone();

        let handle = thread::spawn(move || {
            let mut progress: f64 = 0.0;
            loop {
                thread::sleep(Duration::from_millis(100));

                if stop_clone.load(Ordering::Relaxed) {
                    break;
                }

                if reset.swap(false, Ordering::Relaxed) {
                    progress = 0.0;
                }

                progress += 0.01;
                progress = progress.min(1_f64);

                tx.send(event::Event::Progress(progress)).unwrap();
            }
        });

        self.stop_flag = Some(stop);
        self.handle = Some(handle);
    }

    pub fn handle_paste(&mut self, _text: &str) -> Action {
        Action::None
    }

    pub fn on_db_response(&mut self, _response: crate::db::DbResponse) -> Action {
        Action::None
    }

    pub fn on_exit(&mut self) {
        if let Some(ref stop) = self.stop_flag {
            stop.store(true, Ordering::Relaxed);
        }
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
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

impl Drop for DashboardScene {
    fn drop(&mut self) {
        self.on_exit();
    }
}
