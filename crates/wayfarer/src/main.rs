mod show;


use anyhow::Result;
use clap::Parser as ArgParser;


#[derive(Debug, ArgParser)]
pub(crate) struct Args {
    #[command(subcommand)]
    command: CommandArgs,
}


#[derive(Debug, Clone, clap::Subcommand)]
pub(crate) enum CommandArgs {
    Show(show::Args),
}


fn main() -> Result<()> {
    let args = Args::parse();

    match &args.command {
        CommandArgs::Show(sub_args) => show::execute(&args, sub_args)?,
    }

    Ok(())
}
