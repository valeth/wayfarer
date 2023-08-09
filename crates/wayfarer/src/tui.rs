#![cfg(feature = "tui")]

use std::fs::File;
use std::io::{self, Stdout};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, TryRecvError};
use std::time::Duration;

use anyhow::Result;
use clap::Parser as ArgParser;
use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, execute};
use jrny_save::{Savefile, LEVEL_NAMES};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::widgets::{Block, Borders, Cell, Padding, Paragraph, Row, Table};

#[cfg(feature = "watch")]
use crate::watcher::FileWatcher;
use crate::Args as AppArgs;


#[derive(Debug, Clone, ArgParser)]
pub struct Args {
    path: PathBuf,
}


struct State {
    current_path: PathBuf,
    current_file: Savefile,
    #[cfg(feature = "watch")]
    file_watcher: Option<FileWatcher>,
}


#[derive(Debug, Clone)]
#[non_exhaustive]
enum Message {
    Exit,

    #[cfg(feature = "watch")]
    ToggleFileWatch,

    #[cfg(feature = "watch")]
    ReloadFile,
}


type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;
type Frame<'a> = ratatui::Frame<'a, CrosstermBackend<Stdout>>;


pub(crate) fn execute(_app_args: &AppArgs, args: &Args) -> Result<()> {
    let mut terminal = setup()?;

    let savefile = load_savefile(&args.path)?;

    // TODO: prompt file path
    let state = State {
        current_path: args.path.clone(),
        current_file: savefile,
        #[cfg(feature = "watch")]
        file_watcher: None,
    };

    run(&mut terminal, state)?;

    reset(terminal)?;

    Ok(())
}


#[allow(unused_mut)]
fn run(terminal: &mut Terminal, mut state: State) -> Result<()> {
    let (mut evq_tx, evq_rx) = mpsc::channel::<Message>();

    loop {
        terminal.draw(|frame| {
            render(&state, frame);
        })?;

        handle_events(&mut evq_tx)?;

        let message = match evq_rx.try_recv() {
            Ok(msg) => msg,
            Err(TryRecvError::Empty) => continue,
            Err(_) => break,
        };

        match message {
            Message::Exit => break,

            #[cfg(feature = "watch")]
            Message::ToggleFileWatch => {
                if state.file_watcher.is_none() {
                    let evq_tx = evq_tx.clone();
                    let callback = move || {
                        evq_tx.send(Message::ReloadFile).unwrap();
                    };
                    let file_watcher = FileWatcher::new(&state.current_path, callback);
                    state.file_watcher = Some(file_watcher);
                } else {
                    state.file_watcher = None;
                }
            }

            #[cfg(feature = "watch")]
            Message::ReloadFile => {
                let savefile = load_savefile(&state.current_path)?;
                state.current_file = savefile;
            }
        }
    }

    Ok(())
}


fn handle_events(event_queue: &mut mpsc::Sender<Message>) -> Result<()> {
    if !event::poll(Duration::from_millis(250))? {
        return Ok(());
    }

    match event::read()? {
        Event::Key(key) => handle_keyboard_input(key, event_queue)?,
        _ => return Ok(()),
    }

    Ok(())
}


fn handle_keyboard_input(key: KeyEvent, event_queue: &mut mpsc::Sender<Message>) -> Result<()> {
    let message = match key.code {
        KeyCode::Char('q') => Message::Exit,

        #[cfg(feature = "watch")]
        KeyCode::Char('w') => Message::ToggleFileWatch,

        _ => return Ok(()),
    };

    event_queue.send(message)?;

    Ok(())
}


fn setup() -> Result<Terminal> {
    let mut stdout = io::stdout();

    enable_raw_mode()?;

    execute!(stdout, EnterAlternateScreen)?;

    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}


fn reset(mut terminal: Terminal) -> Result<()> {
    disable_raw_mode()?;

    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    terminal.show_cursor()?;

    Ok(())
}


fn load_savefile<P>(path: P) -> Result<Savefile>
where
    P: AsRef<Path>,
{
    let file = File::open(path)?;
    let savefile = Savefile::from_reader(file)?;

    Ok(savefile)
}


fn render(state: &State, mut frame: &mut Frame) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(40), Constraint::Length(2)])
        .split(frame.size());

    render_info(&state.current_file, &mut frame, rows[0]);

    let status_block = Block::default().padding(Padding::horizontal(2));

    #[cfg(feature = "watch")]
    let text = format!(
        "{} file: {}",
        if state.file_watcher.is_some() {
            "Watching"
        } else {
            "Showing"
        },
        state.current_path.display()
    );
    #[cfg(not(feature = "watch"))]
    let text = format!("Showing file: {}", state.current_path.display());

    let status = Paragraph::new(text).block(status_block);
    frame.render_widget(status, rows[1])
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
