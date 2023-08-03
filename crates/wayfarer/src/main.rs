use std::fs::File;
use std::path::{Path, PathBuf};

use anyhow::Result;
use clap::Parser as ArgParser;
use jrny_save::Savefile;


#[derive(Debug, ArgParser)]
struct Args {
    #[cfg(feature = "watch")]
    #[arg(long, default_value_t = false)]
    watch: bool,

    #[command(subcommand)]
    command: Command,

    path: PathBuf,
}


#[derive(Debug, Clone, clap::Subcommand)]
enum Command {
    Show,
}


fn main() -> Result<()> {
    let args = Args::parse();

    if !args.path.exists() {
        anyhow::bail!("Could not find file at given path");
    }

    match args.command {
        Command::Show => {
            show_all_info(&args.path)?;

            #[cfg(feature = "watch")]
            if args.watch {
                watch_all_info(&args.path)?;
            }
        }
    }

    Ok(())
}


#[cfg(feature = "watch")]
fn watch_all_info<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    use std::sync::mpsc;

    use notify::event::{AccessKind, AccessMode};
    use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    println!("Watching file for changes...");
    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?;

    let written_and_closed = EventKind::Access(AccessKind::Close(AccessMode::Write));

    loop {
        let event = rx.recv()?;

        if event?.kind == written_and_closed {
            show_all_info(&path).unwrap();
        }
    }
}


fn show_all_info<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let file = File::open(path)?;
    let savefile = Savefile::from_reader(file)?;

    println!("------======::::: WAYFARER :::::======------\n");
    general_info(&savefile);
    println!("\n---===---===---===---===---===---===---===---\n");
    current_journey(&savefile);
    println!("\n---===---===---===---===---===---===---===---\n");
    glyphs(&savefile);
    println!("\n---===---===---===---===---===---===---===---\n");
    murals(&savefile);
    println!("\n---===---===---===---===---===---===---===---\n");
    current_companions(&savefile);
    println!("\n---===---===---===---===---===---===---===---\n");
    past_companions(&savefile);
    println!("\n------======::::::::::::::::::::======------");

    Ok(())
}

fn general_info(savefile: &Savefile) {
    println!("Journeys Completed: {:>}", savefile.journey_count);
    println!("Total Companions Met: {:>}", savefile.total_companions_met);
    println!(
        "Total Symbols Collected: {}",
        savefile.total_collected_symbols
    );
}

fn current_journey(savefile: &Savefile) {
    println!("Current Level: {:<10}", savefile.current_level_name());
    println!("Companions Met: {:<10}", savefile.companions_met);
    println!("Scarf Length: {:<10}", savefile.scarf_length);
    println!("Symbol Number: {:<10}", savefile.symbol);
    println!(
        "Robe: {:<10}, Tier {}",
        savefile.robe_color(),
        savefile.robe_tier()
    );
    println!("Last Played: {:<10}", savefile.last_played);
}

fn current_companions(savefile: &Savefile) {
    for companion in savefile.current_companions() {
        println!(
            "{:24} {}",
            companion.name,
            companion.steam_url().to_string()
        );
    }
}

fn past_companions(savefile: &Savefile) {
    for companion in savefile.past_companions() {
        println!(
            "{:24} {}",
            companion.name,
            companion.steam_url().to_string()
        );
    }
}

fn glyphs(savefile: &Savefile) {
    for (level, glyphs) in savefile.glyphs.all() {
        print!("{:<16} ", jrny_save::LEVEL_NAMES[level]);
        for glyph in glyphs {
            print!("{:3}", if glyph { "X" } else { "O" });
        }
        println!();
    }
}

fn murals(savefile: &Savefile) {
    for (level, murals) in savefile.murals.all() {
        print!("{:<16} ", jrny_save::LEVEL_NAMES[level]);
        for mural in murals {
            print!("{:3}", if mural { "X" } else { "O" });
        }
        println!();
    }
}
