pub mod companions;
pub mod glyphs;
pub mod murals;
pub mod stats;


use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

use crate::tui::view::Frame;
use crate::tui::State;


pub(super) fn render(state: &mut State, frame: &mut Frame, area: Rect) {
    if state.is_savefile_loaded() {
        render_info(state, frame, area);
    } else {
        render_no_active_file(frame, area);
    }
}


fn render_no_active_file(frame: &mut Frame, area: Rect) {
    let info_block = Block::default()
        .padding(Padding::horizontal(2))
        .borders(Borders::ALL);

    let info = Paragraph::new("No active file.\nPress 'o' to open a file, or 'q' to quit.")
        .block(info_block);

    frame.render_widget(info, area);
}


fn render_info(state: &mut State, mut frame: &mut Frame, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let left_column = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(columns[0]);

    let right_column = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(10, 10)])
        .split(columns[1]);

    stats::render(state, &mut frame, left_column[0]);
    glyphs::render(state, &mut frame, left_column[1]);
    murals::render(state, &mut frame, left_column[2]);
    companions::render(state, &mut frame, right_column[0]);
}
