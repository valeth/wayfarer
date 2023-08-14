use std::fs::create_dir_all;
use std::path::PathBuf;
#[cfg(feature = "tui")]
use std::{
    fs::{self, read_to_string},
    io::Write,
    os::unix::prelude::OsStrExt,
    path::Path,
};

use anyhow::Result;
use directories::ProjectDirs;
#[cfg(feature = "tui")]
use jrny_save::Savefile;


lazy_static::lazy_static! {
    static ref DIRS: ProjectDirs = {
        ProjectDirs::from("", "valeth", "wayfarer").unwrap()
    };
}


pub fn logs_dir() -> Result<PathBuf> {
    let log_root_path = DIRS
        .state_dir()
        .unwrap_or_else(|| DIRS.cache_dir())
        .join("logs");

    if !log_root_path.exists() {
        create_dir_all(&log_root_path)?;
    }

    Ok(log_root_path)
}


#[cfg(feature = "tui")]
#[derive(Debug, Default)]
pub struct PersistentState {
    pub savefile: Option<Savefile>,
}

#[cfg(feature = "tui")]
impl PersistentState {
    pub fn load() -> Result<Self> {
        let data_dir = DIRS.data_local_dir();

        if !data_dir.exists() {
            create_dir_all(&data_dir)?;
        }

        let savefile = load_last_active_savefile()?;

        Ok(Self { savefile })
    }

    #[cfg(feature = "watch")]
    pub fn reload_active_savefile(&mut self) -> Result<()> {
        if let Some(cur_savefile) = &self.savefile {
            let new_savefile = Savefile::from_path(&cur_savefile.path)?;
            self.savefile = Some(new_savefile);
        }

        Ok(())
    }

    pub fn set_active_savefile_path<P>(&mut self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let savefile = Savefile::from_path(&path)?;

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
}


#[cfg(feature = "tui")]
fn load_last_active_savefile() -> Result<Option<Savefile>> {
    let state_path = DIRS.data_local_dir().join("active_savefile");

    if !state_path.exists() {
        return Ok(None);
    }

    let path = read_to_string(&state_path).unwrap();

    let savefile = Savefile::from_path(path.trim_end())?;

    Ok(Some(savefile))
}
