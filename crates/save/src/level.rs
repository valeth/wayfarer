use core::fmt;

use binrw::{BinRead, BinWrite};

use crate::{Error, Result};


pub const MAX_LEVEL_ID: u64 = 11;

pub const NAMES: [&str; MAX_LEVEL_ID as usize + 1] = [
    "Chapter Select",
    "Broken Bridge",
    "Pink Desert",
    "Sunken City",
    "Underground",
    "Tower",
    "Snow",
    "Paradise",
    "Credits",
    "Level Bryan",
    "Level Matt",
    "Level Chris",
];


#[derive(Debug, Clone, Copy, BinRead, BinWrite)]
pub struct Level {
    #[br(assert(id <= MAX_LEVEL_ID))]
    id: u64,
}

impl Level {
    pub fn set_by_id(&mut self, id: u64) -> Result<()> {
        if id > MAX_LEVEL_ID {
            return Err(Error::LevelIdOutOfRange);
        }

        self.id = id;

        Ok(())
    }

    pub fn set_by_name(&mut self, name: &str) -> Result<()> {
        let id = NAMES
            .iter()
            .position(|&v| v == name)
            .ok_or(Error::LevelNameNotFound)?;

        self.id = id as u64;

        Ok(())
    }

    pub fn wrapping_next(&self) -> Self {
        let id = self.id + 1;
        if id > MAX_LEVEL_ID {
            Self { id: 0 }
        } else {
            Self { id }
        }
    }

    pub fn wrapping_previous(&self) -> Self {
        match self.id.checked_sub(1) {
            Some(id) => Self { id },
            None => Self { id: MAX_LEVEL_ID },
        }
    }
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", NAMES[self.id as usize])
    }
}

impl AsRef<u64> for Level {
    fn as_ref(&self) -> &u64 {
        &self.id
    }
}
