mod edit;
mod show;
mod tui;
mod watcher;


use anyhow::Result;
use clap::Parser as ArgParser;


#[derive(Debug, ArgParser)]
#[command(author, version, about)]
pub(crate) struct Args {
    #[command(subcommand)]
    command: CommandArgs,
}


#[derive(Debug, Clone, clap::Subcommand)]
pub(crate) enum CommandArgs {
    /// Display info about save files
    Show(show::Args),

    /// Edit verious aspect of save files
    Edit(edit::Args),

    #[cfg(feature = "tui")]
    Tui(tui::Args),
}


fn main() -> Result<()> {
    let args = Args::parse();


    match &args.command {
        CommandArgs::Show(sub_args) => show::execute(&args, sub_args)?,

        CommandArgs::Edit(sub_args) => edit::execute(&args, sub_args)?,

        #[cfg(feature = "tui")]
        CommandArgs::Tui(sub_args) => tui::execute(&args, sub_args)?,
    }

    Ok(())
}
