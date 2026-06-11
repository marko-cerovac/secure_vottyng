pub mod dashboard;
pub mod input_cert;
pub mod login;
pub mod register;

use std::sync::mpsc;

use crate::db::{DbRequest, DbResponse};
use crossterm::event::KeyEvent;
use dashboard::DashboardScene;
use input_cert::InputCertScene;
use login::LoginScene;
use ratatui::Frame;
use register::RegisterScene;

use self::Scene::*;
use crate::event;

pub enum Action {
    None,
    SwitchScene(Scene),
    Quit,
}

pub enum Scene {
    InputCert(InputCertScene),
    Login(LoginScene),
    Register(RegisterScene),
    Dashboard(DashboardScene),
}

impl Scene {
    pub fn draw(&self, frame: &mut Frame) {
        match self {
            InputCert(s) => s.draw(frame),
            Register(s) => s.draw(frame),
            Login(s) => s.draw(frame),
            Dashboard(s) => s.draw(frame),
        }
    }

    /// This function provides input processing logic
    /// for each scene sepparately.
    /// This allows a scene to have it's own local keymappings.
    ///
    /// It takes a KeyEvent of the key that was pressed, processes it
    /// and returns an Action to tell the App what to do next.
    pub fn handle(&mut self, key: KeyEvent) -> Action {
        match self {
            InputCert(s) => s.handle(key),
            Register(s) => s.handle(key),
            Login(s) => s.handle(key),
            Dashboard(s) => s.handle(key),
        }
    }

    pub fn handle_paste(&mut self, text: &str) -> Action {
        match self {
            InputCert(s) => s.handle_paste(text),
            Register(s) => s.handle_paste(text),
            Login(s) => s.handle_paste(text),
            Dashboard(s) => s.handle_paste(text),
        }
    }

    pub fn on_db_response(&mut self, response: DbResponse) -> Action {
        match self {
            InputCert(s) => s.on_db_response(response),
            Register(s) => s.on_db_response(response),
            Login(s) => s.on_db_response(response),
            Dashboard(s) => s.on_db_response(response),
        }
    }

    pub fn on_enter(
        &mut self,
        tx: mpsc::Sender<event::Event>,
        db: std::sync::mpsc::Sender<DbRequest>,
    ) {
        match self {
            InputCert(s) => s.on_enter(tx, db),
            Register(s) => s.on_enter(tx, db),
            Login(s) => s.on_enter(tx, db),
            Dashboard(s) => s.on_enter(tx, db),
        }
    }

    pub fn on_exit(&mut self) {
        match self {
            InputCert(s) => s.on_exit(),
            Register(s) => s.on_exit(),
            Login(s) => s.on_exit(),
            Dashboard(s) => s.on_exit(),
        }
    }
}
