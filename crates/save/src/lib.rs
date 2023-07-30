mod companion;
mod glyphs;
mod murals;
mod test;


use core::fmt;
use std::io::Read;

use binrw::{BinRead, BinReaderExt};
use chrono::{DateTime, NaiveDateTime, Utc};

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


#[derive(Debug, PartialEq, Eq)]
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


#[binrw::binread]
#[derive(Debug)]
#[brw(little, magic = b"\x16\x00\x00\x80\x00\x00\x00\x00")]
pub struct Savefile {
    pub robe: u32,

    pub symbol: u32,

    pub scarf_length: u32,

    #[brw(count = 4)]
    _unknown1: Vec<u8>,

    #[brw(assert(current_level <= 12))]
    pub current_level: u64,

    pub total_collected_symbols: u32,

    #[brw(assert(collected_symbols <= 21))]
    pub collected_symbols: u32,

    pub murals: Murals,

    #[brw(count = 22)]
    _unknown2: Vec<u8>,

    #[brw(parse_with = parse_last_played)]
    pub last_played: DateTime<Utc>,

    #[brw(count = 4)]
    _unknown3: Vec<u8>,

    pub journey_count: u64,

    pub glyphs: Glyphs,

    #[brw(count = 2404)]
    _unknown4: Vec<u8>,

    pub companion_symbols: CompanionSymbols,

    pub companions_met: u32,

    #[brw(count = 1024)]
    _unknown6: Vec<u8>,

    pub total_companions_met: u32,

    #[brw(count = 24)]
    _unknown7: Vec<u8>,

    pub companions: Companions,
}

impl Savefile {
    pub fn from_reader<R>(mut reader: R) -> std::io::Result<Self>
    where
        R: Read + BinReaderExt,
    {
        // TODO: implement error type
        Ok(reader.read_le().unwrap())
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

    pub fn robe_tier(&self) -> u32 {
        match self.robe_color() {
            RobeColor::Red => self.robe + 1,
            RobeColor::White => self.robe - 2,
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
