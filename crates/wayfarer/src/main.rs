mod edit;
mod show;
mod state;
mod tui;
mod watcher;


use anyhow::Result;
use clap::Parser as ArgParser;
use tracing::Level as TracingLevel;
use tracing_appender::rolling;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::state::logs_dir;


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
    tracing_setup()?;

    let args = Args::parse();

    match &args.command {
        CommandArgs::Show(sub_args) => show::execute(&args, sub_args)?,

        CommandArgs::Edit(sub_args) => edit::execute(&args, sub_args)?,

        #[cfg(feature = "tui")]
        CommandArgs::Tui(sub_args) => tui::execute(&args, sub_args)?,
    }

    Ok(())
}


fn tracing_setup() -> Result<()> {
    // lower log leven when targeting release
    let level = if cfg!(debug_assertions) {
        TracingLevel::TRACE
    } else {
        TracingLevel::INFO
    };

    let filter_layer = Targets::new()
        .with_target("wayfarer", level)
        .with_default(TracingLevel::ERROR);

    let file_writer = rolling::daily(logs_dir()?, "app");

    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false);

    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(filter_layer)
        .try_init()?;

    Ok(())
}
