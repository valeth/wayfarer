use core::fmt;
use std::fs::{self, create_dir_all, read_to_string};
use std::io::Write;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::{bail, Context, Result};
use jrny_save::Savefile;
use ratatui::widgets::TableState;
use tracing::{debug, error};
use tui_input::Input;

use super::view::info::glyphs::TABLE_RANGE as GLYPHS_TABLE_RANGE;
use super::view::info::murals::TABLE_RANGE as MURALS_TABLE_RANGE;
use super::view::info::stats::TABLE_RANGE as STATS_TABLE_RANGE;
use super::Direction;
#[cfg(feature = "watch")]
use crate::watcher::FileWatcher;
use crate::DIRS;


#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,

    Edit,

    Insert,

    SelectFile,
}

impl Mode {
    pub fn is_editing(&self) -> bool {
        self == &Self::Edit || self == &Self::Insert
    }
}


#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    #[default]
    General,
    Glyphs,
    Murals,
    Companions,
}


#[derive(Default)]
pub struct State {
    pub savefile: Option<Savefile>,
    original_file: Option<Savefile>,
    pub active_section: Section,
    pub stats_table: TableState,
    pub glyphs_table: TableState,
    pub murals_table: TableState,
    pub error_msg: Option<(Instant, String)>,
    pub mode: Mode,
    pub edit_input: Option<Input>,
    pub file_select: Input,
    #[cfg(feature = "watch")]
    file_watcher: Option<FileWatcher>,
}


impl State {
    const ERROR_MSG_DURATION: Duration = Duration::new(3, 0);

    pub fn show_error_message<S>(&mut self, msg: S)
    where
        S: fmt::Display,
    {
        let until = Instant::now() + Self::ERROR_MSG_DURATION;
        error!(%msg);
        self.error_msg = Some((until, msg.to_string()));
    }

    pub fn clear_expired_error_message(&mut self) {
        if let Some((until, _)) = self.error_msg {
            if Instant::now() >= until {
                self.error_msg = None;
            }
        }
    }

    pub fn clear_error_message(&mut self) {
        self.error_msg.take();
    }

    pub fn load() -> Result<Self> {
        let data_dir = DIRS.data_local_dir();

        if !data_dir.exists() {
            create_dir_all(&data_dir)?;
        }

        let savefile = load_last_active_savefile()?;

        Ok(Self {
            savefile,
            ..Default::default()
        })
    }

    pub fn set_savefile_from_path<P>(&mut self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let savefile = Savefile::from_path(path)?;
        self.savefile = Some(savefile);

        Ok(())
    }

    pub fn set_selected_as_active_savefile(&mut self) -> Result<()> {
        let savefile = Savefile::from_path(&self.file_select.value())?;

        let state_path = DIRS.data_local_dir().join("active_savefile");
        let mut state_file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .open(state_path)?;

        let active_savefile = savefile.path.as_os_str().as_bytes();

        state_file.write_all(active_savefile)?;
        self.savefile = Some(savefile);

        Ok(())
    }

    pub fn edit_current_file(&mut self) {
        if !self.mode.is_editing() {
            self.original_file = self.savefile.clone();
            self.select_section(self.active_section);
            self.mode = Mode::Edit;
        }
    }

    pub fn start_editing_entry(&mut self) {
        self.mode = Mode::Insert;
    }

    #[tracing::instrument(skip_all)]
    pub fn commit_entry_edit(&mut self) -> Result<()> {
        debug!(section = ?self.active_section);

        match self.active_section {
            Section::General => self.edit_stats_section()?,
            _ => (),
        }

        self.mode = Mode::Edit;

        Ok(())
    }

    fn edit_stats_section(&mut self) -> Result<()> {
        let Some(savefile) = &mut self.savefile else {
            bail!("No savefile loaded");
        };

        let input = self.edit_input.take().context("no edit input")?;
        let value = input.value();

        match self.stats_table.selected().context("no selection")? {
            0 => savefile.journey_count = value.parse()?,
            1 => savefile.total_companions_met = value.parse()?,
            2 => savefile.total_collected_symbols = value.parse()?,
            3 => savefile.current_level.set_by_name(value)?,
            4 => savefile.companions_met = value.parse()?,
            5 => savefile.scarf_length.set_length(value.parse()?)?,
            6 => savefile.symbol.set_by_id(value.parse()?)?,
            7 => savefile.robe.set_color(value.parse()?),
            8 => savefile.robe.set_tier(value.parse()?)?,
            9 => {}
            idx => debug!("unknown index {:?}", idx),
        }

        Ok(())
    }

    pub fn next_entry_value(&mut self) -> Result<()> {
        match self.active_section {
            Section::General => self.next_stats_entry_value()?,
            _ => (),
        }

        Ok(())
    }

    fn next_stats_entry_value(&mut self) -> Result<()> {
        let Some(savefile) = &mut self.savefile else {
            bail!("No savefile loaded");
        };

        match self.stats_table.selected().context("no selection")? {
            0 => savefile.journey_count += 1,
            1 => savefile.total_companions_met += 1,
            2 => savefile.total_collected_symbols += 1,
            3 => savefile.current_level = savefile.current_level.wrapping_next(),
            4 => savefile.companions_met += 1,
            5 => savefile.scarf_length.increase_length()?,
            6 => savefile.symbol = savefile.symbol.wrapping_next(),
            7 => savefile.robe.swap_colors(),
            8 => savefile.robe.increase_tier(),
            9 => {}
            idx => debug!("unknown index {:?}", idx),
        }

        Ok(())
    }

    pub fn previous_entry_value(&mut self) -> Result<()> {
        match self.active_section {
            Section::General => self.previous_stats_entry_value()?,
            _ => (),
        }

        Ok(())
    }

    fn previous_stats_entry_value(&mut self) -> Result<()> {
        let Some(savefile) = &mut self.savefile else {
            bail!("No savefile loaded");
        };

        match self.stats_table.selected().context("no selection")? {
            0 => savefile.journey_count = savefile.journey_count.saturating_sub(1),
            1 => savefile.total_companions_met = savefile.total_companions_met.saturating_sub(1),
            2 => {
                savefile.total_collected_symbols =
                    savefile.total_collected_symbols.saturating_sub(1)
            }
            3 => savefile.current_level = savefile.current_level.wrapping_previous(),
            4 => savefile.companions_met = savefile.companions_met.saturating_sub(1),
            5 => savefile.scarf_length.decrease_length()?,
            6 => savefile.symbol = savefile.symbol.wrapping_previous(),
            7 => savefile.robe.swap_colors(),
            8 => savefile.robe.decrease_tier(),
            9 => {}
            idx => debug!("unknown index {:?}", idx),
        }
        Ok(())
    }

    pub fn cancel_editing_entry(&mut self) {
        if self.mode == Mode::Insert {
            self.edit_input = None;
            self.mode = Mode::Edit;
        }
    }

    pub fn is_savefile_loaded(&self) -> bool {
        self.savefile.is_some()
    }

    #[cfg(feature = "watch")]
    pub fn is_watching_file(&self) -> bool {
        self.file_watcher.is_some()
    }

    #[cfg(feature = "watch")]
    pub fn enable_file_watcher<F>(&mut self, callback: F)
    where
        F: Fn() + Send + 'static,
    {
        if let Some(savefile) = &self.savefile {
            let file_watcher = FileWatcher::new(&savefile.path, callback);
            self.file_watcher = Some(file_watcher);
        }
    }

    #[cfg(feature = "watch")]
    pub fn reset_file_watcher(&mut self) {
        self.file_watcher = None;
    }

    pub fn reload_active_savefile(&mut self) -> Result<()> {
        if let Some(cur_savefile) = &self.savefile {
            debug!("Reloading file");
            let new_savefile = Savefile::from_path(&cur_savefile.path)?;
            self.savefile = Some(new_savefile);
        }

        Ok(())
    }

    pub fn move_section(&mut self, direction: Direction) {
        let next_section = match (direction, self.active_section) {
            (Direction::Left, Section::Companions) => Section::General,
            (Direction::Left, _) => Section::Companions,
            (Direction::Right, Section::Companions) => Section::General,
            (Direction::Right, _) => Section::Companions,
            (Direction::Down, Section::General) => Section::Glyphs,
            (Direction::Down, Section::Glyphs) => Section::Murals,
            (Direction::Down, Section::Murals) => Section::General,
            (Direction::Down, section) => section,
            (Direction::Up, Section::General) => Section::Murals,
            (Direction::Up, Section::Murals) => Section::Glyphs,
            (Direction::Up, Section::Glyphs) => Section::General,
            (Direction::Up, section) => section,
        };

        self.select_section(next_section);
        self.active_section = next_section;
    }

    fn select_section(&mut self, section: Section) {
        let table = match section {
            Section::General => &mut self.stats_table,
            Section::Glyphs => &mut self.glyphs_table,
            Section::Murals => &mut self.murals_table,
            _ => return,
        };

        if table.selected().is_none() {
            table.select(Some(0));
        }
    }

    pub fn move_in_current_section(&mut self, direction: Direction) {
        match self.active_section {
            Section::General => {
                select_row_in_range(&mut self.stats_table, direction, STATS_TABLE_RANGE)
            }
            Section::Glyphs => {
                select_row_in_range(&mut self.glyphs_table, direction, GLYPHS_TABLE_RANGE)
            }
            Section::Murals => {
                select_row_in_range(&mut self.murals_table, direction, MURALS_TABLE_RANGE)
            }
            _ => (),
        }
    }
}


fn select_row_in_range(table: &mut TableState, direction: Direction, (min, max): (usize, usize)) {
    match (direction, table.selected()) {
        (Direction::Up, Some(i)) if i <= min => (),
        (Direction::Up, Some(i)) => {
            table.select(Some(i - 1));
        }
        (Direction::Down, Some(i)) if i >= max => (),
        (Direction::Down, Some(i)) => {
            table.select(Some(i + 1));
        }
        _ => (),
    }
}


fn load_last_active_savefile() -> Result<Option<Savefile>> {
    let state_path = DIRS.data_local_dir().join("active_savefile");

    if !state_path.exists() {
        return Ok(None);
    }

    let path = read_to_string(&state_path)?;

    let savefile = Savefile::from_path(path.trim_end())?;

    Ok(Some(savefile))
}
