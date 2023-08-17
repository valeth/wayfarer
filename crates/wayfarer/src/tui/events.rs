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
    match (&state.mode, key.code) {
        (_, KeyCode::Char('q')) if key.modifiers.contains(KeyModifiers::CONTROL) => {
            msg_tx.send(Message::Exit)?;
        }

        (Mode::Insert, KeyCode::Esc) => {
            msg_tx.send(Message::CancelEditEntry)?;
        }

        (_, KeyCode::Esc) => {
            msg_tx.send(Message::SetMode(Mode::Normal))?;
        }

        (Mode::SelectFile, KeyCode::Enter) => {
            if state.prompt_save {
                msg_tx.send(Message::SaveFile)?;
            } else {
                msg_tx.send(Message::LoadFile)?;
            }
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

        (Mode::Normal, KeyCode::Char('r')) => {
            msg_tx.send(Message::ReloadFile)?;
        }

        (Mode::Edit, KeyCode::Char('H')) => {
            msg_tx.send(Message::MoveSection(Direction::Left))?;
        }

        (Mode::Edit, KeyCode::Char('J')) => {
            msg_tx.send(Message::MoveSection(Direction::Down))?;
        }

        (Mode::Edit, KeyCode::Char('K')) => {
            msg_tx.send(Message::MoveSection(Direction::Up))?;
        }

        (Mode::Edit, KeyCode::Char('L')) => {
            msg_tx.send(Message::MoveSection(Direction::Right))?;
        }

        (Mode::Edit, KeyCode::Char('h')) => {
            msg_tx.send(Message::MoveCur(Direction::Left))?;
        }

        (Mode::Edit, KeyCode::Char('j')) => {
            msg_tx.send(Message::MoveCur(Direction::Down))?;
        }

        (Mode::Edit, KeyCode::Char('k')) => {
            msg_tx.send(Message::MoveCur(Direction::Up))?;
        }

        (Mode::Edit, KeyCode::Char('l')) => {
            msg_tx.send(Message::MoveCur(Direction::Right))?;
        }

        (Mode::Edit, KeyCode::Enter) => {
            msg_tx.send(Message::StartEditEntry)?;
        }

        (Mode::Edit, KeyCode::Char('n')) => {
            msg_tx.send(Message::NextEntryValue)?;
        }

        (Mode::Edit, KeyCode::Char('p')) => {
            msg_tx.send(Message::PreviousEntryValue)?;
        }

        (Mode::Edit, KeyCode::Char('s')) => {
            state.prompt_save = true;
            state.file_select = Input::default();
            msg_tx.send(Message::SetMode(Mode::SelectFile))?;
        }

        (Mode::Insert, KeyCode::Enter) => {
            msg_tx.send(Message::CommitEditEntry)?;
        }

        (Mode::Insert, _) => {
            if let Some(input) = &mut state.edit_input {
                input.handle_event(&Event::Key(key));
            }
        }

        (Mode::Normal, KeyCode::Char('e')) => {
            msg_tx.send(Message::SetMode(Mode::Edit))?;
        }

        #[cfg(feature = "watch")]
        (Mode::Normal, KeyCode::Char('w')) => {
            msg_tx.send(Message::ToggleFileWatch)?;
        }

        _ => (),
    };

    state.clear_error_message();

    Ok(())
}
