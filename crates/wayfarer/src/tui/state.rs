use std::fs::{self, create_dir_all, read_to_string};
use std::io::Write;
use std::os::unix::prelude::OsStrExt;
use std::path::Path;

use anyhow::Result;
use jrny_save::Savefile;
use tracing::debug;
use tui_input::Input;

#[cfg(feature = "watch")]
use crate::watcher::FileWatcher;
use crate::DIRS;


#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Normal,

    ShowError(String),

    SelectFile,
}


#[derive(Default)]
pub struct State {
    savefile: Option<Savefile>,
    pub mode: Mode,
    pub file_select: Input,
    #[cfg(feature = "watch")]
    file_watcher: Option<FileWatcher>,
}


impl State {
    pub fn load() -> Result<Self> {
        let data_dir = DIRS.data_local_dir();

        if !data_dir.exists() {
            create_dir_all(&data_dir)?;
        }

        let savefile = load_last_active_savefile()?;

        Ok(Self {
            savefile,
            mode: Mode::default(),
            file_select: Input::default(),
            #[cfg(feature = "watch")]
            file_watcher: None,
        })
    }

    pub fn savefile(&self) -> Option<&Savefile> {
        self.savefile.as_ref()
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
