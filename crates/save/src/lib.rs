mod companion;
mod glyphs;
mod murals;
mod symbol;
mod test;


use core::fmt;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use binrw::{until_eof, BinRead, BinReaderExt, BinWriterExt};
use chrono::{DateTime, NaiveDateTime, Utc};
use symbol::Symbol;

use crate::companion::{CompanionSymbols, CompanionWithId, Companions};
use crate::glyphs::Glyphs;
use crate::murals::Murals;


pub const LEVEL_NAMES: [&str; 12] = [
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


pub type Result<T, E = Error> = std::result::Result<T, E>;


#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to deserialize savefile")]
    DeserializationFailed(binrw::Error),

    #[error("Failed to serialize savefile")]
    SerializationFailed(binrw::Error),

    #[error("Failed to read file")]
    FileReadingFailed(io::Error),
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RobeColor {
    Red,
    White,
}

impl fmt::Display for RobeColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Red => write!(f, "Red"),
            Self::White => write!(f, "White"),
        }
    }
}


#[binrw::binrw]
#[derive(Debug, Clone)]
#[brw(little)]
pub struct Savefile {
    #[brw(ignore)]
    pub path: PathBuf,

    #[br(count = 8)]
    _unknown0: Vec<u8>,

    robe: u32,

    pub symbol: Symbol,

    pub scarf_length: u32,

    #[br(count = 4)]
    _unknown1: Vec<u8>,

    #[br(assert(current_level <= 12))]
    pub current_level: u64,

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

    pub fn current_level_name(&self) -> &'static str {
        LEVEL_NAMES[self.current_level as usize]
    }

    pub fn robe_color(&self) -> RobeColor {
        if self.robe > 3 {
            RobeColor::White
        } else {
            RobeColor::Red
        }
    }

    pub fn set_robe_color(&mut self, color: RobeColor) {
        self.robe = match (self.robe_color(), color) {
            (RobeColor::Red, RobeColor::White) => self.robe + 4,
            (RobeColor::White, RobeColor::Red) => self.robe - 4,
            _ => return,
        }
    }

    pub fn robe_tier(&self) -> u32 {
        match self.robe_color() {
            RobeColor::Red => self.robe + 1,
            RobeColor::White => self.robe - 2,
        }
    }

    pub fn set_robe_tier(&mut self, tier: u32) {
        if tier < 1 || tier > 4 {
            return;
        }

        self.robe = match self.robe_color() {
            RobeColor::Red => tier - 1,
            // There can't be a tier 1 white robe, setting it to the lowers possible tier
            RobeColor::White if tier == 1 => 4,
            RobeColor::White => tier + 2,
        }
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
