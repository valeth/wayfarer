use binrw::{BinRead, BinWrite};


#[derive(Debug, Clone, Copy, BinRead, BinWrite)]
pub struct Murals {
    status_flags: u16,
}

impl Murals {
    const COUNT: [usize; 7] = [1, 1, 2, 2, 1, 1, 2];

    pub fn has_found(&self, level_index: usize, index: usize) -> Option<bool> {
        if level_index > 6 {
            return None;
        }

        if index >= Self::COUNT[level_index] {
            return None;
        }

        let pos = Self::COUNT[0..level_index].iter().sum::<usize>();
        let mask = 0x01 << (pos + index);
        Some((self.status_flags & mask) == mask)
    }

    pub fn all<'a>(&'a self) -> impl Iterator<Item = (usize, Vec<bool>)> + 'a {
        Self::COUNT.iter().enumerate().map(|(level, murals)| {
            let murals = (0..*murals)
                .into_iter()
                .map(|mural| self.has_found(level, mural).unwrap())
                .collect::<Vec<_>>();

            (level, murals)
        })
    }
}
