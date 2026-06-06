pub mod dashboard;
pub mod login;
pub mod register;

use std::sync::mpsc;

use crossterm::event::KeyEvent;
use ratatui::Frame;
use dashboard::DashboardScene;
use login::LoginScene;
use register::RegisterScene;

use self::Scene::*;
use crate::event;

pub enum Action {
    None,
    SwitchScene(Scene),
    Quit,
}

pub enum Scene {
    Login(LoginScene),
    Register(RegisterScene),
    Dashboard(DashboardScene),
}

impl Scene {
    pub fn draw(&self, frame: &mut Frame) {
        match self {
            Register(s) => s.draw(frame),
            Login(s) => s.draw(frame),
            Dashboard(s) => s.draw(frame),
        }
    }

    pub fn handle(&mut self, key: KeyEvent) -> Action {
        match self {
            Register(s) => s.handle(key),
            Login(s) => s.handle(key),
            Dashboard(s) => s.handle(key),
        }
    }

    pub fn on_enter(&mut self, tx: mpsc::Sender<event::Event>) {
        match self {
            Register(s) => s.on_enter(tx),
            Login(s) => s.on_enter(tx),
            Dashboard(s) => s.on_enter(tx),
        }
    }

    pub fn on_exit(&mut self) {
        match self {
            Register(s) => s.on_exit(),
            Login(s) => s.on_exit(),
            Dashboard(s) => s.on_exit(),
        }
    }
}
