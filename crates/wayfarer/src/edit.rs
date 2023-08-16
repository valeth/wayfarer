use std::fs::{self, File};
use std::path::PathBuf;

use anyhow::Result;
use clap::builder::PossibleValuesParser;
use clap::{value_parser, Parser as ArgParser};
use jrny_save::{RobeColor, Savefile, LEVEL_NAMES};

use crate::AppArgs;


#[derive(Debug, Clone, ArgParser)]
pub(crate) struct Args {
    in_path: PathBuf,
    out_path: PathBuf,

    #[arg(long, value_parser = value_parser!(u32).range(1..=32))]
    scarf_length: Option<u32>,

    #[arg(long, value_parser = PossibleValuesParser::new(LEVEL_NAMES))]
    current_level: Option<String>,

    #[arg(long, value_parser = value_parser!(u32).range(0..=21))]
    symbol: Option<u32>,

    #[arg(long, value_parser = PossibleValuesParser::new(["red", "white"]))]
    robe_color: Option<String>,

    /// Sets the robe tier from 1 to 4, white robe always has a minimum of 2
    #[arg(long, value_parser = value_parser!(u32).range(1..=4))]
    robe_tier: Option<u32>,
}


pub(crate) fn execute(_app_args: &AppArgs, sub_args: &Args) -> Result<()> {
    let in_file = File::open(&sub_args.in_path)?;

    let savefile = Savefile::from_reader(in_file)?;

    let new_savefile = edit_file(&savefile, &sub_args)?;

    let out_file = fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&sub_args.out_path)?;

    new_savefile.write(out_file)?;

    Ok(())
}


fn edit_file(cur_savefile: &Savefile, args: &Args) -> Result<Savefile> {
    let mut savefile = cur_savefile.clone();

    if let Some(val) = args.scarf_length {
        savefile.scarf_length = val;
    }

    if let Some(val) = &args.current_level {
        savefile.current_level.set_by_name(&val)?;
    }

    if let Some(val) = args.symbol {
        savefile.symbol.set_by_id(val)?;
    }

    if let Some(color) = &args.robe_color {
        match color.as_ref() {
            "red" => savefile.set_robe_color(RobeColor::Red),
            "white" => savefile.set_robe_color(RobeColor::White),
            _ => (),
        }
    }

    if let Some(tier) = args.robe_tier {
        savefile.set_robe_tier(tier);
    }

    Ok(savefile)
}
