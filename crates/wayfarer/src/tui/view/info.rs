use jrny_save::{Savefile, LEVEL_NAMES};
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Cell, Padding, Paragraph, Row, Table};

use crate::tui::state::{Mode, Section};
use crate::tui::view::Frame;
use crate::tui::State;


pub fn render(state: &mut State, frame: &mut Frame, area: Rect) {
    match state.savefile() {
        Some(savefile) => render_info(savefile, state, frame, area),
        None => render_no_active_file(frame, area),
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


fn render_info(savefile: &Savefile, state: &State, mut frame: &mut Frame, area: Rect) {
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

    render_stats(&savefile, state, &mut frame, left_column[0]);
    render_glyphs(&savefile, state, &mut frame, left_column[1]);
    render_murals(&savefile, state, &mut frame, left_column[2]);
    render_companions(&savefile, state, &mut frame, right_column[0]);
}


fn render_stats<'a>(savefile: &Savefile, state: &State, frame: &mut Frame, area: Rect) {
    let border_style = if state.active_section == Section::General && state.mode == Mode::Edit {
        Style::default().fg(ratatui::style::Color::Blue)
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
    .widths(&[Constraint::Ratio(1, 3), Constraint::Ratio(2, 3)])
    .block(stats_block);

    let cur_symbol_block = Block::default();

    let cur_symbol = Paragraph::new(savefile.symbol.to_string()).block(cur_symbol_block);

    frame.render_widget(stats_section_block, area);
    frame.render_widget(cur_symbol, layout[0]);
    frame.render_widget(table, layout[1]);
}


fn render_companions<'a>(savefile: &Savefile, state: &State, frame: &mut Frame, area: Rect) {
    let border_style = if state.active_section == Section::Companions && state.mode == Mode::Edit {
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


fn render_glyphs<'a>(savefile: &Savefile, state: &State, frame: &mut Frame, area: Rect) {
    const FOUND_SIGN: &str = "◆";
    const NOT_FOUND_SIGN: &str = "◇";

    let border_style = if state.active_section == Section::Glyphs && state.mode == Mode::Edit {
        Style::default().fg(ratatui::style::Color::Blue)
    } else {
        Style::default()
    };

    let block = Block::default()
        .title("Glyphs")
        .border_style(border_style)
        .borders(Borders::ALL)
        .padding(Padding::new(2, 2, 1, 1));

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
    .block(block);

    frame.render_widget(table, area);
}


fn render_murals<'a>(savefile: &Savefile, state: &State, frame: &mut Frame, area: Rect) {
    const FOUND_SIGN: &str = "▾";
    const NOT_FOUND_SIGN: &str = "▿";

    let border_style = if state.active_section == Section::Murals && state.mode == Mode::Edit {
        Style::default().fg(ratatui::style::Color::Blue)
    } else {
        Style::default()
    };

    let block = Block::default()
        .title("Murals")
        .border_style(border_style)
        .borders(Borders::ALL)
        .padding(Padding::new(2, 2, 1, 1));

    let table = Table::new(savefile.murals.all().map(|(level_number, status)| {
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
    .block(block);

    frame.render_widget(table, area);
}
