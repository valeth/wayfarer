mod companion;
mod glyphs;
mod level;
mod murals;
mod robe;
mod scarf;
mod symbol;
mod test;


use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use binrw::{until_eof, BinRead, BinReaderExt, BinWriterExt};
use chrono::{DateTime, NaiveDateTime, Utc};
use level::Level;
use robe::Robe;
use scarf::Scarf;
use symbol::Symbol;

use crate::companion::{CompanionSymbols, CompanionWithId, Companions};
use crate::glyphs::Glyphs;
pub use crate::level::NAMES as LEVEL_NAMES;
use crate::murals::Murals;
pub use crate::robe::Color as RobeColor;


pub type Result<T, E = Error> = std::result::Result<T, E>;


#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to deserialize savefile")]
    DeserializationFailed(binrw::Error),

    #[error("Failed to serialize savefile")]
    SerializationFailed(binrw::Error),

    #[error("Level id is out of range")]
    LevelIdOutOfRange,

    #[error("Level name was not found")]
    LevelNameNotFound,

    #[error("Scarf already at maximum length")]
    ScarfMaxLength,

    #[error("Scarf already at minimum length")]
    ScarfMinLength,

    #[error("Scarf can be at most 30 long")]
    ScarfTooLong,

    #[error("Symbol id is out of range")]
    SymbolIdOutOfRange,

    #[error(transparent)]
    RobeChange(robe::Error),

    #[error("Failed to read file")]
    FileReadingFailed(io::Error),
}


#[binrw::binrw]
#[derive(Debug, Clone)]
#[brw(little)]
pub struct Savefile {
    #[brw(ignore)]
    pub path: PathBuf,

    #[br(count = 8)]
    _unknown0: Vec<u8>,

    pub robe: Robe,

    pub symbol: Symbol,

    pub scarf_length: Scarf,

    #[br(count = 4)]
    _unknown1: Vec<u8>,

    pub current_level: Level,

    pub total_collected_symbols: u32,

    #[br(assert(collected_symbols <= 21))]
    pub collected_symbols: u32,

    pub murals: Murals,

    #[br(count = 22)]
    _unknown2: Vec<u8>,

    #[br(parse_with = parse_last_played)]
    #[bw(write_with = write_last_played)]
    pub last_played: DateTime<Utc>,

    #[br(count = 4)]
    _unknown3: Vec<u8>,

    pub journey_count: u64,

    pub glyphs: Glyphs,

    #[br(count = 2404)]
    _unknown4: Vec<u8>,

    pub companion_symbols: CompanionSymbols,

    pub companions_met: u32,

    #[br(count = 1024)]
    _unknown6: Vec<u8>,

    pub total_companions_met: u32,

    #[br(count = 24)]
    _unknown7: Vec<u8>,

    pub companions: Companions,

    #[br(parse_with = until_eof)]
    _unknown8: Vec<u8>,
}

impl Savefile {
    pub fn from_path<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let file = File::open(&path).map_err(Error::FileReadingFailed)?;
        let savefile = Self {
            path: path.as_ref().to_owned(),
            ..Self::from_reader(file)?
        };

        Ok(savefile)
    }

    pub fn from_reader<R>(mut reader: R) -> Result<Self>
    where
        R: Read + BinReaderExt,
    {
        Ok(reader.read_le().map_err(Error::DeserializationFailed)?)
    }

    pub fn write<W>(&self, mut writer: W) -> Result<()>
    where
        W: Write + BinWriterExt,
    {
        writer.write_le(self).map_err(Error::SerializationFailed)?;

        Ok(())
    }

    pub fn current_companions<'a>(&'a self) -> impl Iterator<Item = &'a CompanionWithId> {
        self.companions
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                if idx < self.companions_met as usize {
                    Some(item)
                } else {
                    None
                }
            })
    }

    pub fn past_companions<'a>(&'a self) -> impl Iterator<Item = &'a CompanionWithId> {
        self.companions
            .iter()
            .enumerate()
            .filter_map(|(idx, item)| {
                if idx >= self.companions_met as usize {
                    Some(item)
                } else {
                    None
                }
            })
    }
}

#[binrw::parser(reader, endian)]
fn parse_last_played() -> binrw::BinResult<DateTime<Utc>> {
    let timestamp: i64 = <_>::read_options(reader, endian, ())?;
    let timestamp = (timestamp / 10_000) - 11_644_473_600_000;

    let datetime = NaiveDateTime::from_timestamp_millis(timestamp).unwrap();
    let datetime = DateTime::from_utc(datetime, Utc);

    Ok(datetime)
}

#[binrw::writer(writer, endian)]
fn write_last_played(datetime: &DateTime<Utc>) -> binrw::BinResult<()> {
    let timestamp = datetime.timestamp_millis();
    let timestamp = (timestamp + 11_644_473_600_000) * 10_000;

    writer.write_type(&timestamp, endian)?;

    Ok(())
}
