mod app;
mod event;

use app::App;
use event::Event;
use std::{io, sync::mpsc, thread, time::Duration};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init(); // initialize the terminal in raw mode
    let mut app: App = App::new();
    let (event_tx, event_rx) = mpsc::channel::<Event>();

    let input_events_tx = event_tx.clone();
    let progress_events_tx = event_tx.clone();

    thread::spawn(move || {
        handle_input_events(input_events_tx);
    });

    thread::spawn(move || {
        run_background_thread(progress_events_tx);
    });

    let app_result = app.run(&mut terminal, event_rx);

    ratatui::restore(); // restore the terminal to it's normal state
    app_result
}

fn handle_input_events(tx: mpsc::Sender<event::Event>) {
    loop {
        if let crossterm::event::Event::Key(key_event) = crossterm::event::read().unwrap() {
            tx.send(event::Event::Input(key_event)).unwrap();
        }
    }
}

fn run_background_thread(tx: mpsc::Sender<event::Event>) {
    let mut progress: f64 = 0.0;
    let increment: f64 = 0.01;

    loop {
        thread::sleep(Duration::from_millis(100));
        progress += increment;
        progress = progress.min(1_f64);

        tx.send(event::Event::Progress(progress)).unwrap();
    }
}
