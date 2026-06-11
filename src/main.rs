mod app;
mod crypto;
mod db;
mod event;
mod models;
mod scene;
mod services;

use app::App;
use crossterm::event::{DisableBracketedPaste, EnableBracketedPaste};
use event::Event;
use std::{io, sync::mpsc, thread};

fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();

    let mut terminal = ratatui::init();
    let _ = crossterm::execute!(io::stdout(), EnableBracketedPaste);
    let (event_tx, event_rx) = mpsc::channel::<Event>();

    let input_tx = event_tx.clone();
    thread::spawn(move || {
        loop {
            match crossterm::event::read().unwrap() {
                crossterm::event::Event::Key(key_event) => {
                    input_tx.send(Event::Input(key_event)).unwrap();
                }
                crossterm::event::Event::Paste(text) => {
                    input_tx.send(Event::Paste(text)).unwrap();
                }
                _ => {}
            }
        }
    });

    let tick_tx = event_tx.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(std::time::Duration::from_millis(50));
            if tick_tx.send(Event::Tick).is_err() {
                break;
            }
        }
    });

    let db_req_tx = db::spawn_db_worker(event_tx.clone());
    let mut app = App::new(event_tx, db_req_tx);
    let app_result = app.run(&mut terminal, event_rx);

    let _ = crossterm::execute!(io::stdout(), DisableBracketedPaste);
    ratatui::restore();
    app_result
}
