use binrw::BinRead;


#[derive(Debug, BinRead)]
pub struct Glyphs(#[brw(count = 6)] Vec<LevelGlyphs>);

impl Glyphs {
    const COUNT: [usize; 6] = [3, 3, 4, 3, 4, 4];

    pub fn all<'a>(&'a self) -> impl Iterator<Item = (usize, Vec<bool>)> + 'a {
        self.0.iter().enumerate().map(|(level, glyphs)| {
            let glyphs = (0..Self::COUNT[level])
                .into_iter()
                .map(|glyph_idx| glyphs.has_collected(glyph_idx).unwrap())
                .collect();

            (level, glyphs)
        })
    }

    pub fn count(&self) -> usize {
        self.0.len()
    }

    pub fn has_collected(&self, level: usize, index: usize) -> Option<bool> {
        if level >= self.0.len() {
            return None;
        }

        self.0[level].has_collected(index)
    }
}


#[derive(Debug, BinRead)]
pub struct LevelGlyphs {
    status_flags: u8,

    #[brw(count = 343)]
    _unused: Vec<u8>,
}

impl LevelGlyphs {
    pub fn has_collected(&self, index: usize) -> Option<bool> {
        if index > 8 {
            return None;
        }

        Some(((self.status_flags >> index) & 0x01) == 0x01)
    }
}
