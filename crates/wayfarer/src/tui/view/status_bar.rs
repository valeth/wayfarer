use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::widgets::{Block, Padding, Paragraph};

use crate::tui::view::Frame;
use crate::tui::{Mode, State};


pub fn render(state: &State, mut frame: &mut Frame, area: Rect) {
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
