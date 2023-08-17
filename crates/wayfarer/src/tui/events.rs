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
    if key.modifiers.is_empty() {
        match (key.code, &state.mode) {
            (KeyCode::Esc, Mode::Insert) => msg_tx.send(Message::CancelEditEntry)?,
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
            (KeyCode::Char('q'), Mode::Normal) => msg_tx.send(Message::Exit)?,
            (KeyCode::Char('e'), Mode::Normal) => msg_tx.send(Message::SetMode(Mode::Edit))?,

            // file loading
            (KeyCode::Char('o'), Mode::Normal) => {
                state.prompt_save = false;
                state.file_select = Input::default();
                msg_tx.send(Message::SetMode(Mode::SelectFile))?
            }
            (KeyCode::Char('r'), Mode::Normal) => msg_tx.send(Message::ReloadFile)?,
            #[cfg(feature = "watch")]
            (KeyCode::Char('w'), Mode::Normal) => msg_tx.send(Message::ToggleFileWatch)?,

            // movement inside editor section
            (KeyCode::Char('h'), Mode::Edit) => msg_tx.send(Message::MoveCur(Direction::Left))?,
            (KeyCode::Char('j'), Mode::Edit) => msg_tx.send(Message::MoveCur(Direction::Down))?,
            (KeyCode::Char('k'), Mode::Edit) => msg_tx.send(Message::MoveCur(Direction::Up))?,
            (KeyCode::Char('l'), Mode::Edit) => msg_tx.send(Message::MoveCur(Direction::Right))?,

            // movement between editor sections
            (KeyCode::Char('H'), Mode::Edit) => {
                msg_tx.send(Message::MoveSection(Direction::Left))?
            }
            (KeyCode::Char('J'), Mode::Edit) => {
                msg_tx.send(Message::MoveSection(Direction::Down))?
            }
            (KeyCode::Char('K'), Mode::Edit) => msg_tx.send(Message::MoveSection(Direction::Up))?,
            (KeyCode::Char('L'), Mode::Edit) => {
                msg_tx.send(Message::MoveSection(Direction::Right))?
            }

            // next/previous value selection
            (KeyCode::Char('n'), Mode::Edit) => msg_tx.send(Message::NextEntryValue)?,
            (KeyCode::Char('p'), Mode::Edit) => msg_tx.send(Message::PreviousEntryValue)?,

            // open save prompt
            (KeyCode::Char('s'), Mode::Edit) => {
                state.prompt_save = true;
                state.file_select = Input::default();
                msg_tx.send(Message::SetMode(Mode::SelectFile))?;
            }
            (_, Mode::Insert) => {
                if let Some(input) = &mut state.edit_input {
                    input.handle_event(&Event::Key(key));
                }
            }
            (_, Mode::SelectFile) => {
                state.file_select.handle_event(&Event::Key(key));
            }
            _ => (),
        };
    } else if key.modifiers.contains(KeyModifiers::CONTROL) {
        match (key.code, &state.mode) {
            (KeyCode::Char('q'), _) => msg_tx.send(Message::Exit)?,
            _ => (),
        }
    }

    state.clear_error_message();

    Ok(())
}
