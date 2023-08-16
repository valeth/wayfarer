use core::fmt;

use binrw::{BinRead, BinWrite};

use crate::{Error, Result};


const MAX_LENGTH: u32 = 30;


#[derive(Debug, Clone, BinRead, BinWrite)]
pub struct Scarf {
    #[br(assert(length <= MAX_LENGTH))]
    length: u32,
}

impl Scarf {
    pub fn set_length(&mut self, length: u32) -> Result<()> {
        if length > MAX_LENGTH {
            return Err(Error::ScarfTooLong);
        }

        self.length = length;

        Ok(())
    }

    pub fn increase_length(&mut self) -> Result<()> {
        let length = self.length + 1;
        if length > MAX_LENGTH {
            return Err(Error::ScarfMaxLength);
        }

        self.length = length;

        Ok(())
    }

    pub fn decrease_length(&mut self) -> Result<()> {
        let length = self.length.checked_sub(1).ok_or(Error::ScarfMinLength)?;

        self.length = length;

        Ok(())
    }
}

impl fmt::Display for Scarf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.length)
    }
}

impl AsRef<u32> for Scarf {
    fn as_ref(&self) -> &u32 {
        &self.length
    }
}
