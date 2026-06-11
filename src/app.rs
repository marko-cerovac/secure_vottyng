use std::io;

use crossterm::event::{KeyCode, KeyModifiers};
use std::sync::mpsc;

use ratatui::DefaultTerminal;

use crate::db::DbRequest;
use crate::event::Event;
use crate::scene::{Action, Scene};

pub struct App {
    pub exit: bool,
    current_scene: Scene,
    event_tx: mpsc::Sender<Event>,
    db_req_tx: mpsc::Sender<DbRequest>,
}

impl App {
    pub fn new(event_tx: mpsc::Sender<Event>, db_req_tx: mpsc::Sender<DbRequest>) -> Self {
        App {
            exit: false,
            current_scene: Scene::InputCert(crate::scene::input_cert::InputCertScene::new()),
            event_tx,
            db_req_tx,
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<Event>,
    ) -> io::Result<()> {
        self.current_scene
            .on_enter(self.event_tx.clone(), self.db_req_tx.clone());
        terminal.draw(|frame| self.draw(frame))?;

        while !self.exit {
            match rx.recv().unwrap() {
                Event::Input(key_event) => {
                    self.handle_key_event(key_event)?;
                }
                Event::Paste(text) => {
                    self.current_scene.handle_paste(&text);
                }
                Event::Progress(progress) => {
                    if let Scene::Dashboard(ref mut dash) = self.current_scene {
                        dash.progress = progress;
                    }
                }
                Event::DbResponse(response) => {
                    let action = self.current_scene.on_db_response(response);
                    match action {
                        Action::None => {}
                        Action::SwitchScene(new_scene) => {
                            let mut new_scene = new_scene;
                            self.current_scene.on_exit();
                            new_scene.on_enter(self.event_tx.clone(), self.db_req_tx.clone());
                            self.current_scene = new_scene;
                        }
                        Action::Quit => {
                            self.current_scene.on_exit();
                            self.exit = true;
                        }
                    }
                }
                Event::Tick => {}
            }

            terminal.draw(|frame| self.draw(frame))?;
        }

        self.current_scene.on_exit();
        Ok(())
    }

    fn draw(&self, frame: &mut ratatui::Frame) {
        self.current_scene.draw(frame);
    }

    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> io::Result<()> {
        if key.kind == crossterm::event::KeyEventKind::Press
            && key.modifiers == KeyModifiers::CONTROL
            && key.code == KeyCode::Char('c')
        {
            self.current_scene.on_exit();
            self.exit = true;
            return Ok(());
        }

        if key.kind == crossterm::event::KeyEventKind::Press
            && key.modifiers == KeyModifiers::CONTROL
            && key.code == KeyCode::Char('v')
        {
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                if let Ok(text) = clipboard.get_text() {
                    self.current_scene.handle_paste(&text);
                }
            }
            return Ok(());
        }

        let action = self.current_scene.handle(key);
        match action {
            Action::None => {}
            Action::SwitchScene(new_scene) => {
                let mut new_scene = new_scene;
                self.current_scene.on_exit();
                new_scene.on_enter(self.event_tx.clone(), self.db_req_tx.clone());
                self.current_scene = new_scene;
            }
            Action::Quit => {
                self.current_scene.on_exit();
                self.exit = true;
            }
        }

        Ok(())
    }
}
