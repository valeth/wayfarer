use std::io::SeekFrom;

use binrw::{BinRead, NullString};


#[derive(Debug)]
pub struct Companions(Vec<CompanionWithId>);

impl Companions {
    pub fn iter(&self) -> std::slice::Iter<CompanionWithId> {
        self.0.iter()
    }

    pub fn count(&self) -> usize {
        self.0.len()
    }
}

impl BinRead for Companions {
    type Args<'a> = ();

    fn read_options<R>(
        reader: &mut R,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self>
    where
        R: std::io::Read + std::io::Seek,
    {
        let mut companions = Vec::new();

        loop {
            let mut marker = [0u8; 4];
            reader.seek(SeekFrom::Current(28))?;
            reader.read_exact(&mut marker)?;
            reader.seek(SeekFrom::Current(-32))?;

            if marker != [0x01, 0x00, 0x10, 0x01] {
                break;
            }

            let companion: CompanionWithId = <_>::read_options(reader, endian, ())?;
            reader.seek(SeekFrom::Current(4))?;

            companions.push(companion);
        }

        Ok(Self(companions))
    }
}


#[derive(Debug)]
pub struct CompanionSymbols(Vec<CompanionWithSymbol>);

impl CompanionSymbols {
    pub fn iter(&self) -> std::slice::Iter<CompanionWithSymbol> {
        self.0.iter()
    }

    pub fn count(&self) -> usize {
        self.0.len()
    }
}

impl BinRead for CompanionSymbols {
    type Args<'a> = ();

    fn read_options<R>(
        reader: &mut R,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<Self>
    where
        R: std::io::Read + std::io::Seek,
    {
        let mut companions = Vec::new();

        loop {
            let companion: CompanionWithSymbol = <_>::read_options(reader, endian, ())?;

            if companion.name.is_empty() {
                reader.seek(SeekFrom::Current(-60))?;
                break;
            }

            companions.push(companion);
        }

        let padding = 960 - (companions.len() * 60);
        reader.seek(SeekFrom::Current(padding as i64))?;

        Ok(Self(companions))
    }
}


#[derive(Debug, BinRead)]
pub struct CompanionWithId {
    #[brw(pad_size_to = 24, map = |raw: NullString| raw.to_string())]
    pub name: String,

    #[brw(assert(steam_id != 0))]
    pub steam_id: u32,
}

impl CompanionWithId {
    pub fn steam_id_v3(&self) -> String {
        format!("[U:1:{}]", self.steam_id)
    }

    pub fn steam_url(&self) -> String {
        let steam_id = self.steam_id_v3();
        let encoded_steam_id = urlencoding::encode(&steam_id);
        format!("https://steamcommunity.com/profiles/{}", encoded_steam_id)
    }
}


#[derive(Debug, BinRead)]
pub struct CompanionWithSymbol {
    #[brw(pad_size_to = 52, map = |raw: NullString| raw.to_string())]
    pub name: String,

    #[brw(count = 4)]
    _unknown1: Vec<u8>,

    #[brw(assert((0..=21).contains(&symbol)))]
    pub symbol: u32,
}
