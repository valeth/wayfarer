use core::fmt;
use std::str::FromStr;

use binrw::{BinRead, BinWrite};

use crate::Result;


const MIN_TIER: u32 = 1;
const MAX_TIER: u32 = 4;
const MAX_RED_TIER_ID: u32 = 3;


#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("Tier must be in range from 1 to 4")]
    TierOutOfRange,

    #[error("White tier can not be lower than 2")]
    WhiteTierMinimum,

    #[error("Invalid color, expected red or white")]
    InvalidColor,
}


#[derive(Debug, Clone, BinRead, BinWrite)]
pub struct Robe {
    value: u32,
}

impl Robe {
    pub fn color(&self) -> Color {
        if self.value > MAX_RED_TIER_ID {
            Color::White
        } else {
            Color::Red
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.value = match (self.color(), color, self.value) {
            (Color::Red, Color::White, 0) => MAX_RED_TIER_ID + 1,
            (Color::Red, Color::White, val) => val + MAX_RED_TIER_ID,
            (Color::White, Color::Red, val) => val - MAX_RED_TIER_ID,
            _ => return,
        }
    }

    pub fn swap_colors(&mut self) {
        let new_color = match self.color() {
            Color::Red => Color::White,
            Color::White => Color::Red,
        };

        self.set_color(new_color);
    }

    pub fn tier(&self) -> u32 {
        match self.color() {
            Color::Red => self.value + 1,
            Color::White => self.value - MAX_RED_TIER_ID + 1,
        }
    }

    pub fn set_tier(&mut self, tier: u32) -> Result<(), Error> {
        if tier < MIN_TIER || tier > MAX_TIER {
            return Err(Error::TierOutOfRange);
        }

        self.value = match self.color() {
            Color::Red => tier - 1,
            Color::White if tier == MIN_TIER => {
                return Err(Error::WhiteTierMinimum);
            }
            Color::White => MAX_RED_TIER_ID + tier - 1,
        };

        Ok(())
    }

    pub fn increase_tier(&mut self) {
        let _ = self.set_tier(self.tier() + 1);
    }

    pub fn decrease_tier(&mut self) {
        let _ = self.set_tier(self.tier() - 1);
    }
}

impl fmt::Display for Robe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    White,
}

impl FromStr for Color {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Red" | "red" => Ok(Self::Red),
            "White" | "white" => Ok(Self::White),
            _ => Err(Error::InvalidColor),
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Red => write!(f, "Red"),
            Self::White => write!(f, "White"),
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn change_robe_color() {
        // lowest red tier
        let mut robe = Robe { value: 0 };

        assert_eq!(robe.color(), Color::Red);
        assert_eq!(robe.tier(), 1);

        robe.set_color(Color::White);

        assert_eq!(robe.color(), Color::White);
        assert_eq!(robe.tier(), 2);

        robe.set_color(Color::Red);

        assert_eq!(robe.color(), Color::Red);
        assert_eq!(robe.tier(), 2);

        // highest red tier
        let mut robe = Robe {
            value: MAX_RED_TIER_ID,
        };

        assert_eq!(robe.color(), Color::Red);
        assert_eq!(robe.tier(), 4);

        robe.set_color(Color::White);

        assert_eq!(robe.color(), Color::White);
        assert_eq!(robe.tier(), 4);

        robe.set_color(Color::Red);

        assert_eq!(robe.color(), Color::Red);
        assert_eq!(robe.tier(), 4);
    }


    #[test]
    fn change_red_robe_tier() {
        let mut robe = Robe { value: 0 };

        for tier in 1..=4 {
            robe.set_tier(tier).unwrap();
            assert_eq!(robe.tier(), tier, "unexpected tier");
            assert_eq!(robe.color(), Color::Red, "unexpected color");
        }
    }

    #[test]
    fn change_white_robe_tier() {
        let mut robe = Robe { value: 4 };

        let result = robe.set_tier(1);
        assert_eq!(result, Err(Error::WhiteTierMinimum));

        for (tier, expected) in (2..=4).zip([2, 3, 4]) {
            robe.set_tier(tier).unwrap();
            assert_eq!(robe.tier(), expected, "unexpected tier");
            assert_eq!(robe.color(), Color::White, "unexpected color");
        }
    }
}
