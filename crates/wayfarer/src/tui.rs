#![cfg(feature = "tui")]

mod events;
mod state;
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

use self::state::{Mode, State};
use crate::Args as AppArgs;


type Terminal = ratatui::Terminal<CrosstermBackend<Stdout>>;


#[derive(Debug, Clone, ArgParser)]
pub struct Args;


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


pub(crate) fn execute(_app_args: &AppArgs, _args: &Args) -> Result<()> {
    let state = State::load()?;

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

            state.set_selected_as_active_savefile()?;

            #[cfg(feature = "watch")]
            if state.is_watching_file() {
                state.reset_file_watcher();
            }

            msg_tx.send(Message::SetMode(Mode::Normal))?;
        }

        #[cfg(feature = "watch")]
        Message::ToggleFileWatch => {
            if let Some(savefile) = state.savefile() {
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

        #[cfg(feature = "watch")]
        Message::ReloadFile => {
            state.reload_active_savefile()?;
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
