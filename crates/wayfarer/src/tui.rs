#![cfg(feature = "tui")]

use std::io::{self, Stdout};
use std::sync::mpsc::{self, TryRecvError};
use std::time::Duration;

use anyhow::Result;
use clap::Parser as ArgParser;
use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, execute};
use jrny_save::{Savefile, LEVEL_NAMES};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::widgets::{Block, Borders, Cell, Padding, Paragraph, Row, Table};
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use crate::state::PersistentState;
#[cfg(feature = "watch")]
use crate::watcher::FileWatcher;
use crate::Args as AppArgs;


#[derive(Debug, Clone, ArgParser)]
pub struct Args;


struct State {
    persistent: PersistentState,
    mode: Mode,
    file_select: Input,
    #[cfg(feature = "watch")]
    file_watcher: Option<FileWatcher>,
}


#[derive(Debug, Clone)]
#[non_exhaustive]
enum Message {
    Exit,

    SetMode(Mode),

    LoadFile,

    #[cfg(feature = "watch")]
    ToggleFileWatch,

    #[cfg(feature = "watch")]
    ReloadFile,
}


#[derive(Debug, Default, Clone, PartialEq, Eq)]
enum Mode {
    #[default]
    Normal,

    ShowError(String),

    SelectFile,
}


type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<Stdout>>;


pub(crate) fn execute(_app_args: &AppArgs, _args: &Args) -> Result<()> {
    let persistent = PersistentState::load()?;

    let mut terminal = setup()?;

    let state = State {
        persistent,
        mode: Mode::default(),
        file_select: Input::default(),
        #[cfg(feature = "watch")]
        file_watcher: None,
    };

    run(&mut terminal, state)?;

    reset(terminal)?;

    Ok(())
}


#[cfg_attr(not(feature = "watch"), allow(unused_mut))]
fn run(terminal: &mut Terminal, mut state: State) -> Result<()> {
    let (mut msg_tx, msg_rx) = mpsc::channel::<Message>();

    loop {
        terminal.draw(|frame| {
            render(&mut state, frame);
        })?;

        handle_events(&mut msg_tx, &mut state)?;

        match msg_rx.try_recv() {
            Ok(Message::Exit) => break,
            Ok(message) => {
                if let Err(err) = handle_message(&mut state, &mut msg_tx, message) {
                    state.mode = Mode::ShowError(format!("{}", err));
                }
            }
            Err(TryRecvError::Empty) => (),
            Err(_) => break,
        };
    }

    Ok(())
}


#[cfg_attr(not(feature = "watch"), allow(unused_variables))]
fn handle_message(
    state: &mut State,
    msg_tx: &mut mpsc::Sender<Message>,
    message: Message,
) -> Result<()> {
    match message {
        Message::SetMode(mode) => {
            state.mode = mode;
        }

        Message::LoadFile => {
            state
                .persistent
                .set_active_savefile_path(state.file_select.value())?;

            #[cfg(feature = "watch")]
            if state.file_watcher.is_some() {
                state.file_watcher = None;
            }

            msg_tx.send(Message::SetMode(Mode::Normal))?;
        }

        #[cfg(feature = "watch")]
        Message::ToggleFileWatch if state.persistent.savefile.is_some() => {
            let savefile = state.persistent.savefile.as_ref().unwrap();

            if state.file_watcher.is_none() {
                let evq_tx = msg_tx.clone();
                let callback = move || {
                    evq_tx.send(Message::ReloadFile).unwrap();
                };
                let file_watcher = FileWatcher::new(&savefile.path, callback);
                state.file_watcher = Some(file_watcher);
            } else {
                state.file_watcher = None;
            }
        }

        #[cfg(feature = "watch")]
        Message::ReloadFile if state.persistent.savefile.is_some() => {
            state.persistent.reload_active_savefile()?;
        }

        _ => (),
    }

    Ok(())
}


fn handle_events(event_queue: &mut mpsc::Sender<Message>, state: &mut State) -> Result<()> {
    if !event::poll(Duration::from_millis(250))? {
        return Ok(());
    }

    match event::read()? {
        Event::Key(key) => handle_keyboard_input(key, event_queue, state)?,
        _ => return Ok(()),
    }

    Ok(())
}


fn handle_keyboard_input(
    key: KeyEvent,
    msg_tx: &mut mpsc::Sender<Message>,
    state: &mut State,
) -> Result<()> {
    match (&state.mode, key.code) {
        (_, KeyCode::Char('q')) if key.modifiers.contains(KeyModifiers::CONTROL) => {
            msg_tx.send(Message::Exit)?;
        }

        (_, KeyCode::Esc) => {
            msg_tx.send(Message::SetMode(Mode::Normal))?;
        }

        (Mode::ShowError(_), _) => {
            msg_tx.send(Message::SetMode(Mode::Normal))?;
        }

        (Mode::SelectFile, KeyCode::Enter) => {
            msg_tx.send(Message::LoadFile)?;
        }

        (Mode::SelectFile, _) => {
            state.file_select.handle_event(&Event::Key(key));
        }

        (Mode::Normal, KeyCode::Char('q')) => {
            msg_tx.send(Message::Exit)?;
        }

        (Mode::Normal, KeyCode::Char('o')) => {
            msg_tx.send(Message::SetMode(Mode::SelectFile))?;
        }

        #[cfg(feature = "watch")]
        (Mode::Normal, KeyCode::Char('w')) => {
            msg_tx.send(Message::ToggleFileWatch)?;
        }

        _ => (),
    };

    Ok(())
}


fn setup() -> Result<Terminal> {
    let mut stdout = io::stdout();

    enable_raw_mode()?;

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}


fn reset(mut terminal: Terminal) -> Result<()> {
    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    terminal.show_cursor()?;

    Ok(())
}


fn render(state: &mut State, mut frame: &mut Frame) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(40), Constraint::Length(2)])
        .split(frame.size());

    match &state.persistent.savefile {
        Some(savefile) => render_info(&savefile, &mut frame, rows[0]),
        None => render_no_active_file(&mut frame, rows[0]),
    }

    render_status_bar(&state, &mut frame, rows[1]);
}


fn render_no_active_file(frame: &mut Frame, area: Rect) {
    let info_block = Block::default()
        .padding(Padding::horizontal(2))
        .borders(Borders::ALL);

    let info = Paragraph::new("No active file.\nPress 'o' to open a file, or 'q' to quit.")
        .block(info_block);

    frame.render_widget(info, area);
}


fn render_status_bar(state: &State, mut frame: &mut Frame, area: Rect) {
    let status_block = Block::default().padding(Padding::horizontal(2));

    match &state.mode {
        Mode::ShowError(error_msg) => {
            let error_msg = Paragraph::new(error_msg.clone())
                .style(Style::default().fg(ratatui::style::Color::LightRed))
                .block(status_block);
            frame.render_widget(error_msg, area);
        }

        #[cfg(feature = "watch")]
        Mode::Normal if state.file_watcher.is_some() && state.persistent.savefile.is_some() => {
            let savefile = state.persistent.savefile.as_ref().unwrap();
            let text = format!("Watching file: {}", savefile.path.display());
            let status = Paragraph::new(text).block(status_block);
            frame.render_widget(status, area);
        }

        Mode::Normal if state.persistent.savefile.is_none() => (),

        Mode::Normal => {
            let savefile = state.persistent.savefile.as_ref().unwrap();
            let text = format!("Showing file: {}", savefile.path.display());
            let status = Paragraph::new(text).block(status_block);
            frame.render_widget(status, area);
        }

        Mode::SelectFile => render_file_select(&state, &mut frame, status_block, area),
    }
}

fn render_file_select(state: &State, frame: &mut Frame, block: Block, area: Rect) {
    const PROMPT: &str = "Open file:";
    const PADDING: usize = 2;

    let scroll = state.file_select.visual_scroll(area.width as usize);
    let input = Paragraph::new(format!("{} {}", PROMPT, state.file_select.value()))
        .scroll((0, scroll as u16))
        .block(block);
    frame.render_widget(input, area);
    frame.set_cursor(
        area.x + (state.file_select.visual_cursor() + PROMPT.len() + 1 + PADDING) as u16,
        area.y,
    );
}


fn render_info(savefile: &Savefile, mut frame: &mut Frame, area: Rect) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    let left_column = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Ratio(6, 12),
            Constraint::Ratio(3, 12),
            Constraint::Ratio(3, 12),
        ])
        .split(columns[0]);

    let right_column = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Ratio(10, 10)])
        .split(columns[1]);


    render_stats(&savefile, &mut frame, left_column[0]);
    render_glyphs(&savefile, &mut frame, left_column[1]);
    render_murals(&savefile, &mut frame, left_column[2]);

    render_companions(&savefile, &mut frame, right_column[0]);
}


fn render_stats<'a>(savefile: &Savefile, frame: &mut Frame, area: Rect) {
    let stats_section_block = Block::default()
        .padding(Padding::new(2, 2, 1, 1))
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


fn render_companions<'a>(savefile: &Savefile, frame: &mut Frame, area: Rect) {
    let companions_block = Block::default()
        .title("Companions")
        .padding(Padding::new(2, 2, 1, 1))
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


fn render_glyphs<'a>(savefile: &Savefile, frame: &mut Frame, area: Rect) {
    const FOUND_SIGN: &str = "◆";
    const NOT_FOUND_SIGN: &str = "◇";

    let block = Block::default()
        .title("Glyphs")
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


fn render_murals<'a>(savefile: &Savefile, frame: &mut Frame, area: Rect) {
    const FOUND_SIGN: &str = "▾";
    const NOT_FOUND_SIGN: &str = "▿";

    let block = Block::default()
        .title("Murals")
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
