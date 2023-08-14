use std::sync::mpsc;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use tracing::debug;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use super::{Message, Mode, State};


pub fn handle(event_queue: &mut mpsc::Sender<Message>, state: &mut State) -> Result<()> {
    if !event::poll(Duration::from_millis(250))? {
        return Ok(());
    }

    match event::read()? {
        Event::Key(key) => handle_keyboard_input(key, event_queue, state)?,
        Event::Paste(val) => handle_paste(val, state)?,
        _ => return Ok(()),
    }

    Ok(())
}


fn handle_paste(value: String, state: &mut State) -> Result<()> {
    match &state.mode {
        Mode::SelectFile => {
            debug!("Received pasted content: {:?}", value);
            let combined = format!("{}{}", state.file_select.value(), value);
            state.file_select = Input::new(combined);
        }
        _ => (),
    }
    Ok(())
}


#[tracing::instrument(skip(msg_tx, state))]
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
