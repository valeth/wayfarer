mod events;
mod state;
mod view;


use std::io::{self, Stdout};
use std::path::PathBuf;
use std::sync::mpsc::{self, TryRecvError};

use anyhow::Result;
use clap::Parser as ArgParser;
use crossterm::event::{
    DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use tracing::{debug, info};

use self::state::{Mode, State};


type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;


#[derive(Debug, Clone, ArgParser)]
pub struct Args {
    /// Overrides the last loaded file
    #[arg(long, short)]
    path: Option<PathBuf>,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}


#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Message {
    Exit,

    SetMode(Mode),

    LoadFile,

    #[cfg(feature = "watch")]
    ToggleFileWatch,

    StartEditEntry,

    CommitEditEntry,

    CancelEditEntry,

    NextEntryValue,

    PreviousEntryValue,

    ReloadFile,

    MoveSection(Direction),

    MoveCur(Direction),
}


pub(crate) fn execute(args: &Args) -> Result<()> {
    let state = match &args.path {
        Some(path) => {
            let mut state = State::default();
            state.set_savefile_from_path(path)?;
            state
        }
        None => State::load()?,
    };

    let mut terminal = setup()?;

    run(&mut terminal, state)?;

    reset(terminal)?;

    Ok(())
}


#[tracing::instrument(skip_all)]
#[cfg_attr(not(feature = "watch"), allow(unused_mut))]
fn run(terminal: &mut Terminal, mut state: State) -> Result<()> {
    let (mut msg_tx, msg_rx) = mpsc::channel::<Message>();

    loop {
        terminal.draw(|frame| {
            view::render(&mut state, frame);
        })?;

        events::handle(&mut msg_tx, &mut state)?;

        state.clear_expired_error_message();

        match msg_rx.try_recv() {
            Ok(Message::Exit) => {
                debug!("Exiting...");
                break;
            }
            Ok(message) => {
                if let Err(err) = handle_message(&mut state, &mut msg_tx, message) {
                    state.show_error_message(err);
                }
            }
            Err(TryRecvError::Empty) => (),
            Err(_) => break,
        };
    }

    Ok(())
}


#[tracing::instrument(skip(state, msg_tx))]
#[cfg_attr(not(feature = "watch"), allow(unused_variables))]
fn handle_message(
    state: &mut State,
    msg_tx: &mut mpsc::Sender<Message>,
    message: Message,
) -> Result<()> {
    match message {
        Message::SetMode(Mode::Edit) => state.edit_current_file(),
        Message::SetMode(mode) => {
            debug!("Setting mode to {:?}", mode);

            state.mode = mode;
        }
        Message::LoadFile => {
            let file_path = state.file_select.value();
            info!("Loading file {}", file_path);

            state.set_selected_as_active_savefile()?;

            #[cfg(feature = "watch")]
            if state.is_watching_file() {
                state.reset_file_watcher();
            }

            msg_tx.send(Message::SetMode(Mode::Normal))?;
        }
        #[cfg(feature = "watch")]
        Message::ToggleFileWatch => {
            if let Some(savefile) = &state.savefile {
                if state.is_watching_file() {
                    let evq_tx = msg_tx.clone();
                    let callback = move || {
                        evq_tx.send(Message::ReloadFile).unwrap();
                    };

                    info!("Starting file watcher on {}", savefile.path.display());
                    state.enable_file_watcher(callback);
                } else {
                    info!("Stopped file watcher on {}", savefile.path.display());
                    state.reset_file_watcher();
                }
            }
        }
        Message::ReloadFile => state.reload_active_savefile()?,
        Message::MoveSection(direction) => state.move_section(direction),
        Message::MoveCur(direction) => state.move_in_current_section(direction),
        Message::StartEditEntry => state.start_editing_entry(),
        Message::CommitEditEntry => {
            if let Err(err) = state.commit_entry_edit() {
                state.show_error_message(err);
            }
        }
        Message::CancelEditEntry => state.cancel_editing_entry(),
        Message::NextEntryValue => {
            if let Err(err) = state.next_entry_value() {
                state.show_error_message(err);
            }
        }
        Message::PreviousEntryValue => {
            if let Err(err) = state.previous_entry_value() {
                state.show_error_message(err);
            }
        }
        _ => (),
    }

    Ok(())
}


fn setup() -> Result<Terminal> {
    let mut stdout = io::stdout();

    debug!("Enabling terminal raw mode");
    enable_raw_mode()?;

    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )?;

    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}


fn reset(mut terminal: Terminal) -> Result<()> {
    debug!("Disabling terminal raw mode");
    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        DisableBracketedPaste
    )?;

    terminal.show_cursor()?;

    Ok(())
}
