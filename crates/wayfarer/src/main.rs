mod edit;
mod tui;
mod watcher;


use std::fs::create_dir_all;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser as ArgParser;
use directories::ProjectDirs;
use tracing::Level as TracingLevel;
use tracing_appender::rolling;
use tracing_subscriber::filter::Targets;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;


lazy_static::lazy_static! {
    pub static ref DIRS: ProjectDirs = {
        ProjectDirs::from("", "valeth", "wayfarer").unwrap()
    };
}


#[derive(Debug, ArgParser)]
#[command(author, version, about)]
pub(crate) struct AppArgs {
    #[command(flatten)]
    tui_args: tui::Args,

    #[command(subcommand)]
    command: Option<CommandArgs>,
}


#[derive(Debug, Clone, clap::Subcommand)]
pub(crate) enum CommandArgs {
    /// Edit verious aspect of save files
    Edit(edit::Args),
}


fn main() -> Result<()> {
    tracing_setup()?;

    let args = AppArgs::parse();


    match &args.command {
        Some(CommandArgs::Edit(sub_args)) => edit::execute(&args, sub_args)?,
        None => tui::execute(&args.tui_args)?,
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


fn logs_dir() -> Result<PathBuf> {
    let log_root_path = DIRS
        .state_dir()
        .unwrap_or_else(|| DIRS.cache_dir())
        .join("logs");

    if !log_root_path.exists() {
        create_dir_all(&log_root_path)?;
    }

    Ok(log_root_path)
}
