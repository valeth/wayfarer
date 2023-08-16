use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Cell, Padding, Paragraph, Row, Table};
use tui_input::Input;

use crate::tui::state::{Mode, Section};
use crate::tui::view::Frame;
use crate::tui::State;


pub const TABLE_RANGE: (usize, usize) = (0, 9);


pub(super) fn render<'a>(state: &mut State, frame: &mut Frame, area: Rect) {
    let Some(savefile) = &state.savefile else {
        return
    };

    let is_selected = state.active_section == Section::General && state.mode.is_editing();

    let border_style = if is_selected {
        Style::default().fg(Color::Blue)
    } else {
        Style::default()
    };

    let stats_section_block = Block::default()
        .padding(Padding::new(2, 2, 1, 1))
        .border_style(border_style)
        .borders(Borders::ALL);

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
        .split(stats_section_block.inner(area));

    let stats_block = Block::default().title("Stats");

    let table_highlight = if is_selected {
        Style::default().fg(Color::Blue)
    } else {
        Style::default()
    };

    let rows = [
        ("Journeys Completed", savefile.journey_count.to_string()),
        (
            "Total Companions Met",
            savefile.total_companions_met.to_string(),
        ),
        (
            "Total Symbols Collected",
            savefile.total_collected_symbols.to_string(),
        ),
        ("Current Level", savefile.current_level.to_string()),
        ("Companions Met", savefile.companions_met.to_string()),
        ("Scarf Length", savefile.scarf_length.to_string()),
        ("Symbol Number", savefile.symbol.as_ref().to_string()),
        ("Robe Color", savefile.robe.color().to_string()),
        ("Robe Tier", savefile.robe.tier().to_string()),
        ("Last Played", savefile.last_played.to_string()),
    ]
    .into_iter()
    .enumerate()
    .map(|(idx, (title, value))| {
        let value = match state.stats_table.selected() {
            Some(sel) if sel == idx => {
                let value = match state.mode {
                    Mode::Insert => {
                        if state.edit_input.is_none() {
                            state.edit_input.replace(Input::new(value));
                        }

                        let input = state.edit_input.as_ref().unwrap();
                        input.value().to_string()
                    }
                    Mode::Edit => format!("< {} >", value),
                    _ => value.to_string(),
                };

                Cell::from(value)
            }
            _ => Cell::from(value.to_string()),
        };

        Row::new([Cell::from(title), value])
    });

    let table = Table::new(rows)
        .highlight_style(table_highlight)
        .widths(&[Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
        .block(stats_block);

    let cur_symbol_block = Block::default();

    let cur_symbol = Paragraph::new(savefile.symbol.to_string()).block(cur_symbol_block);

    frame.render_widget(stats_section_block, area);
    frame.render_widget(cur_symbol, layout[0]);
    frame.render_stateful_widget(table, layout[1], &mut state.stats_table);

    if state.mode == Mode::Insert {
        if let Some(idx) = state.stats_table.selected() {
            if let Some(input) = &state.edit_input {
                frame.set_cursor(
                    layout[1].x + layout[1].width / 3 + (input.visual_cursor() + 1) as u16,
                    layout[1].y + (idx + 1 - state.stats_table.offset()) as u16,
                );
            }
        }
    }
}
