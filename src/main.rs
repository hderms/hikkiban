use std::io;
use std::io::Read;

use anyhow::anyhow;
use clap::{Parser, Subcommand};
use log::{error, info};
use std::path::PathBuf;

use crate::clipboard::{CopyFileClipboard, get_os_env_target};

mod clipboard;

#[derive(Parser, Debug)]
#[command(name = "cb")]
#[command(
    version,
    about = "A CLI tool for copying and pasting. Supports WSL/WSL2 and OSX primarily",
    long_about = "Clipboard CLI tool for copying/pasting file contents/stdin
"
)]
pub struct Cli {
    /// disable WSL/2 compatibility mode
    #[arg(short, long, global = true, action = clap::ArgAction::SetFalse)]
    pub nowsl: bool,

    /// verbosity level (e.g., -v, -vv, -vvv)
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbosity: u8,

    /// The subcommand to run
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Copy text or file contents to the clipboard
    Copy {
        /// The file path to copy from (if omitted, reads from stdin)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },

    /// Paste contents from the clipboard
    Paste {
        /// The file path to paste into (if omitted, prints to stdout)
        #[arg(short, long)]
        file: Option<PathBuf>,
    },
}

///Convert number of verbosity flags to a log level filter (e.g., -v, -vv, -vvv).
fn log_level_filter(verbosity: u8) -> log::LevelFilter {
    match verbosity {
        0 => log::LevelFilter::Error,
        1 => log::LevelFilter::Info,
        2 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    }
}
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    //Logging
    if cli.verbosity > 0 {
        env_logger::Builder::new()
            .filter_level(log_level_filter(cli.verbosity))
            .init();
    }

    match cli.command {
        Command::Copy { file } => {
            if let Some(file_path) = file {
                info!("Copying from file: {:?}", file_path);
                get_os_env_target(cli.nowsl).copy_file(file_path)?
            } else {
                info!("Copying from stdin...");
                let mut buffer = Vec::new();

                let bytes_read = io::stdin().read_to_end(&mut buffer)?;

                if bytes_read == 0 {
                    error!("No input provided. Nothing copied to clipboard.");
                    return Err(anyhow!("No input provided. Nothing copied to clipboard."));
                }
                clipboard_anywhere::set_clipboard(&String::from_utf8_lossy(&buffer))?;
            }
        }
        Command::Paste { file } => {
            if let Some(file_path) = file {
                info!("Pasting into file: {:?}", file_path);
                std::fs::write(file_path, clipboard_anywhere::get_clipboard()?)?;
            } else {
                info!("Pasting to stdout...");
                let cb = clipboard_anywhere::get_clipboard()?;
                println!("{}", cb);
            }
        }
    }
    Ok(())
}
