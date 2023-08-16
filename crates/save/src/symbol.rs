use core::fmt;

use binrw::{BinRead, BinWrite};
use substring::Substring;

use crate::{Error, Result};


const MAX_SYMBOL_ID: u32 = 20;
const SYMBOL_PARTS: &str = include_str!("symbol_parts.txt");
const SYMBOL_PART_WIDTH: usize = 6;
const SYMBOL_PART_HEIGTH: usize = 3;


#[derive(Debug, Clone, Copy, BinRead, BinWrite)]
pub struct Symbol {
    #[br(assert(id < MAX_SYMBOL_ID))]
    id: u32,
}

impl Symbol {
    pub fn set_by_id(&mut self, id: u32) -> Result<()> {
        if id > MAX_SYMBOL_ID {
            return Err(Error::SymbolIdOutOfRange);
        }

        self.id = id;

        Ok(())
    }

    pub fn wrapping_next(&self) -> Self {
        let id = self.id + 1;
        if id > MAX_SYMBOL_ID {
            Self { id: 0 }
        } else {
            Self { id }
        }
    }

    pub fn wrapping_previous(&self) -> Self {
        match self.id.checked_sub(1) {
            Some(id) => Self { id },
            None => Self { id: MAX_SYMBOL_ID },
        }
    }
}

impl AsRef<u32> for Symbol {
    fn as_ref(&self) -> &u32 {
        &self.id
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", get_symbol(self.id as usize).unwrap())
    }
}

fn get_symbol(id: usize) -> Option<String> {
    let (top_left, top_right, btm_left, btm_right) = symbol_part_ids(id)?;
    get_symbol_with_parts(top_left, top_right, btm_left, btm_right)
}

fn symbol_part_ids(id: usize) -> Option<(usize, usize, usize, usize)> {
    let ids = match id {
        0 => (0, 1, 3, 2),
        1 => (4, 4, 7, 7),
        2 => (9, 9, 13, 2),
        3 => (15, 16, 9, 9),
        4 => (4, 9, 4, 9),
        5 => (15, 12, 3, 9),
        6 => (5, 5, 9, 12),
        7 => (12, 9, 15, 15),
        8 => (7, 9, 12, 8),
        9 => (12, 12, 9, 9),
        10 => (14, 7, 14, 7),
        11 => (8, 8, 13, 13),
        12 => (2, 3, 2, 3),
        13 => (10, 7, 7, 12),
        14 => (7, 7, 10, 12),
        15 => (15, 15, 15, 15),
        16 => (4, 4, 4, 4),
        17 => (11, 10, 11, 10),
        18 => (12, 8, 12, 8),
        19 => (6, 6, 11, 10),
        20 => (12, 9, 11, 10),
        _ => return None,
    };

    Some(ids)
}

fn get_symbol_with_parts(
    top_left: usize,
    top_right: usize,
    btm_left: usize,
    btm_right: usize,
) -> Option<String> {
    if top_left > 16 || top_right > 16 || btm_left > 16 || btm_right > 16 {
        return None;
    }

    let top_left = get_symbol_part(top_left);
    let top_right = get_symbol_part(top_right);
    let btm_left = get_symbol_part(btm_left);
    let btm_right = get_symbol_part(btm_right);

    let top = top_left
        .lines()
        .zip(top_right.lines())
        .map(|(left, right)| format!("{}  {}", left, right));

    let bottom = btm_left
        .lines()
        .zip(btm_right.lines())
        .map(|(left, right)| format!("{}  {}", left, right));

    const EMPTY_LINE: &str = "              ";

    let symbol = top
        .chain([EMPTY_LINE.to_string()].into_iter())
        .chain(bottom)
        .collect::<Vec<_>>()
        .join("\n");

    Some(symbol)
}

fn get_symbol_part(idx: usize) -> &'static str {
    const SYMBOL_PART_SIZE: usize = SYMBOL_PART_WIDTH * SYMBOL_PART_HEIGTH + 2;
    let start = (SYMBOL_PART_SIZE + 2) * idx;
    let end = start + SYMBOL_PART_SIZE;
    SYMBOL_PARTS.substring(start, end)
}
