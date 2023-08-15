use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Padding, Row, Table};

use crate::tui::state::{Mode, Section};
use crate::tui::view::Frame;
use crate::tui::State;


pub(super) fn render<'a>(state: &State, frame: &mut Frame, area: Rect) {
    let Some(savefile) = state.savefile() else {
        return
    };

    let is_selected = state.active_section == Section::Companions && state.mode == Mode::Edit;

    let border_style = if is_selected {
        Style::default().fg(ratatui::style::Color::Blue)
    } else {
        Style::default()
    };

    let companions_block = Block::default()
        .title("Companions")
        .padding(Padding::new(2, 2, 1, 1))
        .border_style(border_style)
        .borders(Borders::ALL);

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(1, 2), Constraint::Ratio(1, 2)])
        .split(companions_block.inner(area));

    let current_companions_block = Block::default()
        .title("Current")
        .borders(Borders::TOP)
        .title_alignment(Alignment::Center);

    let current_companions = Table::new(
        savefile
            .current_companions()
            .map(|companion| Row::new([companion.name.clone(), companion.steam_url()])),
    )
    .widths(&[Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
    .block(current_companions_block);

    let past_companions_block = Block::default()
        .title("Past")
        .borders(Borders::TOP)
        .title_alignment(Alignment::Center);

    let past_companions = Table::new(
        savefile
            .past_companions()
            .map(|companion| Row::new([companion.name.clone(), companion.steam_url()])),
    )
    .widths(&[Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
    .block(past_companions_block);

    frame.render_widget(companions_block, area);
    frame.render_widget(current_companions, layout[0]);
    frame.render_widget(past_companions, layout[1]);
}
