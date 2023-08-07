use std::io::SeekFrom;

use binrw::{BinRead, BinWrite, BinWriterExt, NullString};


#[derive(Debug, Clone)]
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

impl BinWrite for Companions {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        for companion in &self.0 {
            writer.write_type(companion, endian)?;
            writer.write_all(&[0x01, 0x00, 0x10, 0x01])?;
        }

        Ok(())
    }
}


#[derive(Debug, Clone)]
pub struct CompanionSymbols(Vec<CompanionWithSymbol>);

impl CompanionSymbols {
    const ENTRY_SIZE: i64 = 60;
    const SECTION_SIZE: i64 = 960;

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
                reader.seek(SeekFrom::Current(-Self::ENTRY_SIZE))?;
                break;
            }

            companions.push(companion);
        }

        let padding = Self::SECTION_SIZE - (companions.len() as i64 * Self::ENTRY_SIZE);
        reader.seek(SeekFrom::Current(padding))?;

        Ok(Self(companions))
    }
}

impl BinWrite for CompanionSymbols {
    type Args<'a> = ();

    fn write_options<W: std::io::Write + std::io::Seek>(
        &self,
        writer: &mut W,
        endian: binrw::Endian,
        _args: Self::Args<'_>,
    ) -> binrw::BinResult<()> {
        for companion in &self.0 {
            writer.write_type(companion, endian)?;
        }

        let padding = Self::SECTION_SIZE - (self.0.len() as i64 * Self::ENTRY_SIZE);
        writer.write_all(&vec![0u8; padding as usize])?;

        Ok(())
    }
}


#[derive(Debug, Clone, BinRead, BinWrite)]
pub struct CompanionWithId {
    #[br(pad_size_to = 24, map = |raw: NullString| raw.to_string())]
    #[bw(pad_size_to = 24, map = |s| NullString::from(s.as_ref()))]
    pub name: String,

    #[br(assert(steam_id != 0))]
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


#[derive(Debug, Clone, BinRead, BinWrite)]
pub struct CompanionWithSymbol {
    #[br(pad_size_to = 52, map = |raw: NullString| raw.to_string())]
    #[bw(pad_size_to = 52, map = |s| NullString::from(s.as_ref()))]
    pub name: String,

    #[br(count = 4)]
    _unknown1: Vec<u8>,

    #[br(assert((0..=21).contains(&symbol)))]
    pub symbol: u32,
}
