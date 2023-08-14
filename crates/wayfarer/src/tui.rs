#![cfg(feature = "tui")]

mod events;
mod view;


use std::io::{self, Stdout};
use std::sync::mpsc::{self, TryRecvError};

use anyhow::Result;
use clap::Parser as ArgParser;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use tracing::{debug, error, info};
use tui_input::Input;

use crate::state::PersistentState;
#[cfg(feature = "watch")]
use crate::watcher::FileWatcher;
use crate::Args as AppArgs;


type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;


#[derive(Debug, Clone, ArgParser)]
pub struct Args;


pub struct State {
    persistent: PersistentState,
    mode: Mode,
    file_select: Input,
    #[cfg(feature = "watch")]
    file_watcher: Option<FileWatcher>,
}


#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Message {
    Exit,

    SetMode(Mode),

    LoadFile,

    #[cfg(feature = "watch")]
    ToggleFileWatch,

    #[cfg(feature = "watch")]
    ReloadFile,
}


#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,

    ShowError(String),

    SelectFile,
}


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


#[tracing::instrument(skip_all)]
#[cfg_attr(not(feature = "watch"), allow(unused_mut))]
fn run(terminal: &mut Terminal, mut state: State) -> Result<()> {
    let (mut msg_tx, msg_rx) = mpsc::channel::<Message>();

    loop {
        terminal.draw(|frame| {
            view::render(&mut state, frame);
        })?;

        events::handle(&mut msg_tx, &mut state)?;

        match msg_rx.try_recv() {
            Ok(Message::Exit) => {
                debug!("Exiting...");
                break;
            }
            Ok(message) => {
                if let Err(err) = handle_message(&mut state, &mut msg_tx, message) {
                    error!(message = ?err);
                    state.mode = Mode::ShowError(format!("{}", err));
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
        Message::SetMode(mode) => {
            debug!("Setting mode to {:?}", mode);

            state.mode = mode;
        }

        Message::LoadFile => {
            let file_path = state.file_select.value();

            info!("Loading file {}", file_path);

            state.persistent.set_active_savefile_path(file_path)?;

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

                info!("Starting file watcher on {}", savefile.path.display());

                let file_watcher = FileWatcher::new(&savefile.path, callback);
                state.file_watcher = Some(file_watcher);
            } else {
                info!("Stopped file watcher on {}", savefile.path.display());

                state.file_watcher = None;
            }
        }

        #[cfg(feature = "watch")]
        Message::ReloadFile if state.persistent.savefile.is_some() => {
            debug!("Reloading file");

            state.persistent.reload_active_savefile()?;
        }

        _ => (),
    }

    Ok(())
}


fn setup() -> Result<Terminal> {
    let mut stdout = io::stdout();

    debug!("Enabling terminal raw mode");
    enable_raw_mode()?;

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}


fn reset(mut terminal: Terminal) -> Result<()> {
    debug!("Disabling terminal raw mode");
    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    terminal.show_cursor()?;

    Ok(())
}
