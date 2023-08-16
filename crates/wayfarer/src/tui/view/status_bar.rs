use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Padding, Paragraph};

use crate::tui::view::Frame;
use crate::tui::{Mode, State};


pub fn render(state: &State, frame: &mut Frame, area: Rect) {
    let status_block = Block::default().padding(Padding::horizontal(2));

    match &state.error_msg {
        Some((_, msg)) => render_error_message(&msg, frame, status_block, area),
        None => render_status(state, frame, status_block, area),
    }
}

fn render_error_message(msg: &str, frame: &mut Frame, block: Block, area: Rect) {
    let error_msg = Paragraph::new(msg)
        .style(Style::default().fg(ratatui::style::Color::LightRed))
        .block(block);
    frame.render_widget(error_msg, area);
}

pub fn render_status(state: &State, mut frame: &mut Frame, block: Block, area: Rect) {
    match &state.mode {
        Mode::Edit | Mode::Insert => {
            if let Some(savefile) = &state.savefile {
                let text = format!("Editing file: {}", savefile.path.display());
                let status = Paragraph::new(text).block(block);
                frame.render_widget(status, area);
            }
        }

        #[cfg(feature = "watch")]
        Mode::Normal if state.is_watching_file() => {
            if let Some(savefile) = &state.savefile {
                let text = format!("Watching file: {}", savefile.path.display());
                let status = Paragraph::new(text).block(block);
                frame.render_widget(status, area);
            }
        }

        Mode::SelectFile => render_file_select(&state, &mut frame, block, area),

        _ => {
            if let Some(savefile) = &state.savefile {
                let text = format!("Showing file: {}", savefile.path.display());
                let status = Paragraph::new(text).block(block);
                frame.render_widget(status, area);
            }
        }
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
