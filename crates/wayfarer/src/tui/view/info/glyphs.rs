use jrny_save::LEVEL_NAMES;
use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Cell, Padding, Row, Table};

use crate::tui::state::{Mode, Section};
use crate::tui::view::Frame;
use crate::tui::State;


pub const TABLE_RANGE: (usize, usize) = (0, 5);


pub(super) fn render<'a>(state: &mut State, frame: &mut Frame, area: Rect) {
    const FOUND_SIGN: &str = "◆";
    const NOT_FOUND_SIGN: &str = "◇";

    let Some(savefile) = state.savefile() else {
        return
    };

    let is_selected = state.active_section == Section::Glyphs && state.mode == Mode::Edit;


    let border_style = if is_selected {
        Style::default().fg(Color::Blue)
    } else {
        Style::default()
    };

    let block = Block::default()
        .title("Glyphs")
        .border_style(border_style)
        .borders(Borders::ALL)
        .padding(Padding::new(2, 2, 1, 1));

    let table_highlight = if is_selected {
        Style::default().fg(Color::Blue)
    } else {
        Style::default()
    };

    let table = Table::new(savefile.glyphs.all().map(|(level_number, status)| {
        let status = status
            .iter()
            .map(|&val| Cell::from(if val { FOUND_SIGN } else { NOT_FOUND_SIGN }));
        Row::new(
            [Cell::from(LEVEL_NAMES[level_number])]
                .into_iter()
                .chain(status),
        )
    }))
    .widths(&[
        Constraint::Length(20),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(3),
    ])
    .column_spacing(1)
    .highlight_style(table_highlight)
    .block(block);

    frame.render_stateful_widget(table, area, &mut state.glyphs_table);
}
