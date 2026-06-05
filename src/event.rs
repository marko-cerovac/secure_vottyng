pub enum Event {
    Input(crossterm::event::KeyEvent),
    Progress(f64),
}
