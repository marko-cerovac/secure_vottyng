use crate::db::DbResponse;

pub enum Event {
    Input(crossterm::event::KeyEvent),
    Paste(String),
    Progress(f64),
    Tick,
    DbResponse(DbResponse),
}
