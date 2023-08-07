mod edit;
mod show;


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
}


fn main() -> Result<()> {
    let args = Args::parse();

    match &args.command {
        CommandArgs::Show(sub_args) => show::execute(&args, sub_args)?,
        CommandArgs::Edit(sub_args) => edit::execute(&args, sub_args)?,
    }

    Ok(())
}
