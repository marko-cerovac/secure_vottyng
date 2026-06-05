mod app;

use std::io;
use app::App;

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init(); // initialize the terminal in raw mode
    let mut app: App = App::new();

    let app_result = app.run(&mut terminal);

    ratatui::restore(); // restore the terminal to it's normal state
    app_result
}

