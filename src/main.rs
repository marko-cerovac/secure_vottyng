mod app;
mod event;
mod scene;

use app::App;
use event::Event;
use std::{io, sync::mpsc, thread};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let (event_tx, event_rx) = mpsc::channel::<Event>();

    let input_tx = event_tx.clone();
    thread::spawn(move || loop {
        if let crossterm::event::Event::Key(key_event) = crossterm::event::read().unwrap() {
            input_tx.send(Event::Input(key_event)).unwrap();
        }
    });

    let mut app = App::new(event_tx);
    let app_result = app.run(&mut terminal, event_rx);

    ratatui::restore();
    app_result
}
