pub mod info;
pub mod status_bar;


use std::io::Stdout;

use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};

use super::State;


type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<Stdout>>;


pub fn render(state: &mut State, frame: &mut Frame) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(40), Constraint::Length(2)])
        .split(frame.size());

    info::render(state, frame, rows[0]);

    status_bar::render(state, frame, rows[1]);
}
