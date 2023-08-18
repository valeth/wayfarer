use std::sync::mpsc;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use tracing::debug;
use tui_input::backend::crossterm::EventHandler;
use tui_input::Input;

use super::{Direction, Message, Mode, State};


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
    match (key.code, &state.mode) {
        (KeyCode::Esc, Mode::Insert) => msg_tx.send(Message::CancelEditEntry)?,
        (KeyCode::Esc, Mode::SelectFile) => {
            if state.prompt_save {
                msg_tx.send(Message::SetMode(Mode::Edit))?;
            } else {
                msg_tx.send(Message::SetMode(Mode::Normal))?;
            }
        }
        (KeyCode::Esc, _) => msg_tx.send(Message::SetMode(Mode::Normal))?,
        (KeyCode::Enter, Mode::Edit) => msg_tx.send(Message::StartEditEntry)?,
        (KeyCode::Enter, Mode::Insert) => msg_tx.send(Message::CommitEditEntry)?,
        (KeyCode::Enter, Mode::SelectFile) => {
            if state.prompt_save {
                msg_tx.send(Message::SaveFile)?;
            } else {
                msg_tx.send(Message::LoadFile)?;
            }
        }
        (KeyCode::Char('q'), _) if key.modifiers == KeyModifiers::CONTROL => {
            msg_tx.send(Message::Exit)?
        }
        (KeyCode::Char(ch), Mode::Normal) => match ch {
            'q' => msg_tx.send(Message::Exit)?,
            'e' => msg_tx.send(Message::SetMode(Mode::Edit))?,
            'o' => {
                state.prompt_save = false;
                state.file_select = Input::default();
                msg_tx.send(Message::SetMode(Mode::SelectFile))?
            }
            'r' => msg_tx.send(Message::ReloadFile)?,
            #[cfg(feature = "watch")]
            'w' => msg_tx.send(Message::ToggleFileWatch)?,
            _ => (),
        },
        (KeyCode::Char(ch), Mode::Edit) => match ch {
            // movement inside editor section
            'h' => msg_tx.send(Message::MoveCur(Direction::Left))?,
            'j' => msg_tx.send(Message::MoveCur(Direction::Down))?,
            'k' => msg_tx.send(Message::MoveCur(Direction::Up))?,
            'l' => msg_tx.send(Message::MoveCur(Direction::Right))?,
            // movement between editor sections
            'H' => msg_tx.send(Message::MoveSection(Direction::Left))?,
            'J' => msg_tx.send(Message::MoveSection(Direction::Down))?,
            'K' => msg_tx.send(Message::MoveSection(Direction::Up))?,
            'L' => msg_tx.send(Message::MoveSection(Direction::Right))?,
            // next/previous value selection
            'n' => msg_tx.send(Message::NextEntryValue)?,
            'p' => msg_tx.send(Message::PreviousEntryValue)?,
            // open save prompt
            's' => {
                state.prompt_save = true;
                state.file_select = Input::default();
                msg_tx.send(Message::SetMode(Mode::SelectFile))?;
            }
            _ => (),
        },
        (_, Mode::Insert) => {
            if let Some(input) = &mut state.edit_input {
                input.handle_event(&Event::Key(key));
            }
        }
        (_, Mode::SelectFile) => {
            state.file_select.handle_event(&Event::Key(key));
        }
        _ => (),
    }

    state.clear_error_message();

    Ok(())
}
