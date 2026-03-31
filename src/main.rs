use std::io;
use std::os::unix::process::CommandExt;
use std::{error::Error, io::Read};

use anyhow::anyhow;
use clap::{Parser, Subcommand};
use log::{debug, error, info};

/// Clipboard CLI tool for copying any file content that clipboard-rs supports. Also supports WSL2 preferentially if the --wsl2 flag is set, which will use WSL2's clipboard integration instead of the default method. This allows for seamless copying and pasting between Windows and WSL2 environments.
#[derive(Parser, Debug)]
#[command(name = "cb")]
#[command(
    version,
    about = "A CLI tool for copying and pasting. Supports WSL2",
    long_about = "Clipboard CLI tool for copying any file content that clipboard-rs supports. Also supports WSL2 preferentially if the --wsl2 flag is set, which will use WSL2's clipboard integration instead of the default method. This allows for seamless copying and pasting between Windows and WSL2 environments.
"
)]
pub struct Cli {
    /// Enable WSL2 compatibility mode
    #[arg(long, global = true, default_value_t = true)]
    pub wsl2: bool,

    /// verbosity level
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
        file: Option<String>,
    },

    /// Paste contents from the clipboard
    Paste {
        /// The file path to paste into (if omitted, prints to stdout)
        #[arg(short, long)]
        file: Option<String>,
    },
}

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

    if cli.verbosity > 0 {
        env_logger::Builder::new()
            .filter_level(log_level_filter(cli.verbosity))
            .init();
    }

    match cli.command {
        Command::Copy { file } => {
            if let Some(file_path) = file {
                // Copy from the specified file
                info!("Copying from file: {}", file_path);
                // Implement the logic to read the file and copy its contents to the clipboard
                if is_wsl::is_wsl() {
                    debug!("Running in WSL environment, using WSL clipboard integration");
                    copy_file_powershell(&file_path)?;
                } else {
                    let content = std::fs::read_to_string(file_path)?;
                    clipboard_anywhere::set_clipboard(&content)?;
                }
            } else {
                // Copy from stdin
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
                // Paste into the specified file
                info!("Pasting into file: {}", file_path);
                // Implement the logic to paste clipboard contents into the specified file
                std::fs::write(file_path, clipboard_anywhere::get_clipboard()?)?;
            } else {
                // Paste to stdout
                info!("Pasting to stdout...");
                // Implement the logic to paste clipboard contents to stdout
                let cb = clipboard_anywhere::get_clipboard()?;
                println!("{}", cb);
            }
        }
    }
    Ok(())
}

fn copy_file_powershell(file: &str) -> anyhow::Result<()> {
    debug!("Copying file '{}' to clipboard using PowerShell", file);
    duct::cmd!("powershell.exe", "-sta", "Set-Clipboard", "-Path", file).run()?;

    Ok(())
}
