#![cfg(test)]

use std::io::Cursor;

use binrw::BinReaderExt;
use chrono::NaiveDate;

use crate::*;


#[test]
fn general_info() {
    let savefile = savefile();

    assert_eq!(savefile.robe, 3);
    assert_eq!(savefile.robe_color(), RobeColor::Red);
    assert_eq!(savefile.robe_tier(), 4);

    assert_eq!(savefile.symbol.as_ref(), &7);
    assert_eq!(savefile.scarf_length, 27);
    assert_eq!(savefile.current_level, 1);
    assert_eq!(savefile.total_collected_symbols, 107);
    assert_eq!(savefile.collected_symbols, 21);
    assert_eq!(savefile.journey_count, 21);
    assert_eq!(savefile.companions_met, 6);
    assert_eq!(savefile.total_companions_met, 21);
}


#[test]
fn last_played() {
    let savefile = savefile();

    let expected = NaiveDate::from_ymd_opt(2023, 07, 28).unwrap();
    let expected = expected.and_hms_milli_opt(14, 17, 45, 893).unwrap();

    assert_eq!(savefile.last_played.naive_utc(), expected);
}


#[test]
fn companion_info() {
    let savefile = savefile();

    assert_eq!(savefile.companions.count(), 8);
    assert_eq!(savefile.companion_symbols.count(), 8);

    for (a, b) in savefile
        .companions
        .iter()
        .zip(savefile.companion_symbols.iter())
    {
        assert_eq!(a.name, b.name);
    }

    let companion = savefile
        .companion_symbols
        .iter()
        .find(|x| x.name == "Wanderer")
        .unwrap();
    assert_eq!(companion.symbol, 6);
    let companion = savefile
        .companion_symbols
        .iter()
        .find(|x| x.name == "Rythulian")
        .unwrap();
    assert_eq!(companion.symbol, 19);
    let companion = savefile
        .companion_symbols
        .iter()
        .find(|x| x.name == "Machine")
        .unwrap();
    assert_eq!(companion.symbol, 20);
}


#[test]
fn companion_order() {
    let savefile = savefile();

    let current = savefile.current_companions().collect::<Vec<_>>();
    let past = savefile.past_companions().collect::<Vec<_>>();

    assert_eq!(current.len(), 6);
    assert_eq!(past.len(), 2);

    assert_eq!(current[0].name, "Wanderer".to_string());
    assert_eq!(current[1].name, "Rythulian".to_string());
    assert_eq!(past[0].name, "Machine".to_string());
}


#[test]
fn glyph_status() {
    let savefile = savefile();

    const FOUND: [&[bool]; 6] = [
        &[true, false, true],
        &[true, false, false],
        &[true, false, false, true],
        &[true, true, true],
        &[true, false, true, true],
        &[false, true, false, true],
    ];

    for (level_idx, level_found) in FOUND.into_iter().enumerate() {
        for (glyph_idx, has_found) in level_found.into_iter().enumerate() {
            assert_eq!(
                savefile.glyphs.has_collected(level_idx, glyph_idx),
                Some(*has_found),
                "level {}, glyph {} is incorrect",
                level_idx,
                glyph_idx
            );
        }
    }

    assert_eq!(savefile.glyphs.has_collected(0, 69), None);
}


#[test]
fn murals() {
    let savefile = savefile();

    const FOUND: [&[bool]; 7] = [
        &[false],
        &[false],
        &[true, true],
        &[false, true],
        &[false],
        &[true],
        &[true, false],
    ];

    for (level_idx, level_found) in FOUND.into_iter().enumerate() {
        for (mural_idx, has_found) in level_found.into_iter().enumerate() {
            assert_eq!(
                savefile.murals.has_found(level_idx, mural_idx),
                Some(*has_found),
                "level {}, mural {} is incorrect",
                level_idx,
                mural_idx
            )
        }
    }

    assert_eq!(savefile.murals.has_found(69, 420), None);
}


#[test]
fn change_robe_color() {
    let mut savefile = savefile();

    // lowest tier
    savefile.robe = 0;

    savefile.set_robe_color(RobeColor::White);
    assert_eq!(savefile.robe_color(), RobeColor::White);

    savefile.set_robe_color(RobeColor::Red);
    assert_eq!(savefile.robe_color(), RobeColor::Red);

    // highest tier
    savefile.robe = 3;

    savefile.set_robe_color(RobeColor::White);
    assert_eq!(savefile.robe_color(), RobeColor::White);

    savefile.set_robe_color(RobeColor::Red);
    assert_eq!(savefile.robe_color(), RobeColor::Red);
}


#[test]
fn change_robe_tier() {
    let mut savefile = savefile();

    savefile.set_robe_tier(1);
    assert_eq!(savefile.robe_tier(), 1);
    assert_eq!(savefile.robe_color(), RobeColor::Red);

    savefile.set_robe_tier(4);
    assert_eq!(savefile.robe_tier(), 4);
    assert_eq!(savefile.robe_color(), RobeColor::Red);

    savefile.set_robe_color(RobeColor::White);

    savefile.set_robe_tier(2);
    assert_eq!(savefile.robe_tier(), 2);
    assert_eq!(savefile.robe_color(), RobeColor::White);

    savefile.set_robe_tier(1);
    assert_eq!(savefile.robe_tier(), 2);
    assert_eq!(savefile.robe_color(), RobeColor::White);

    savefile.set_robe_tier(4);
    assert_eq!(savefile.robe_tier(), 4);
    assert_eq!(savefile.robe_color(), RobeColor::White);
}


fn savefile() -> Savefile {
    const TEST_FILE: &[u8] = include_bytes!("../test.bin");
    let mut savefile = Cursor::new(TEST_FILE);
    savefile.read_le().expect("parsing failed")
}
