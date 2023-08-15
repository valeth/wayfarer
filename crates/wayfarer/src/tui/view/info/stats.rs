use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Block, Borders, Padding, Paragraph, Row, Table};

use crate::tui::state::{Mode, Section};
use crate::tui::view::Frame;
use crate::tui::State;


pub const TABLE_RANGE: (usize, usize) = (0, 9);


pub(super) fn render<'a>(state: &mut State, frame: &mut Frame, area: Rect) {
    let Some(savefile) = state.savefile() else {
        return
    };

    let is_selected = state.active_section == Section::General && state.mode == Mode::Edit;

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

    let table = Table::new([
        Row::new([
            "Journeys Completed".to_string(),
            savefile.journey_count.to_string(),
        ]),
        Row::new([
            "Total Companions Met".to_string(),
            savefile.total_companions_met.to_string(),
        ]),
        Row::new([
            "Total Symbols Collected".to_string(),
            savefile.total_collected_symbols.to_string(),
        ]),
        Row::new(["Current Level", savefile.current_level_name()]),
        Row::new([
            "Companions Met".to_string(),
            savefile.companions_met.to_string(),
        ]),
        Row::new([
            "Scarf Length".to_string(),
            savefile.scarf_length.to_string(),
        ]),
        Row::new(["Symbol Number".to_string(), savefile.symbol.id.to_string()]),
        Row::new(["Robe Color".to_string(), savefile.robe_color().to_string()]),
        Row::new(["Robe Tier".to_string(), savefile.robe_tier().to_string()]),
        Row::new(["Last Played".to_string(), savefile.last_played.to_string()]),
    ])
    .highlight_style(table_highlight)
    .widths(&[Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
    .block(stats_block);

    let cur_symbol_block = Block::default();

    let cur_symbol = Paragraph::new(savefile.symbol.to_string()).block(cur_symbol_block);

    frame.render_widget(stats_section_block, area);
    frame.render_widget(cur_symbol, layout[0]);
    frame.render_stateful_widget(table, layout[1], &mut state.stats_table);
}
