use std::{fs, path::PathBuf};

use anyhow::anyhow;
use log::debug;

/// Copying a file to the system clipboard.
pub(crate) trait CopyFileClipboard {
    /// Copy the file at `filepath` to the clipboard.
    fn copy_file(&self, filepath: std::path::PathBuf) -> anyhow::Result<()>;
}

/// Target environment / operating system detected at runtime.
///
/// Variants:
/// - OSX: native macOS environment
/// - WSL2: Windows Subsystem for Linux 2
/// - Generic: fallback cross-platform behavior
pub(crate) enum OSEnvTarget {
    OSX,
    WSL2,
    Generic,
}

/// Detect the current operating system or runtime environment.
pub(crate) fn get_os_env_target() -> OSEnvTarget {
    let info = os_info::get();
    if is_wsl::is_wsl() {
        return OSEnvTarget::WSL2;
    } else {
        if info.os_type() == os_info::Type::Macos {
            return OSEnvTarget::OSX;
        } else {
            return OSEnvTarget::Generic;
        }
    }
}

/// Platform-specific implementation of CopyFileClipboard for OSEnvTarget.
impl CopyFileClipboard for OSEnvTarget {
    fn copy_file(&self, filepath: PathBuf) -> anyhow::Result<()> {
        match self {
            OSEnvTarget::OSX => copy_file_osx(filepath),
            OSEnvTarget::WSL2 => copy_file_powershell(filepath),
            OSEnvTarget::Generic => {
                let content = std::fs::read_to_string(filepath)?;
                clipboard_anywhere::set_clipboard(&content)?;
                Ok(())
            }
        }
    }
}

/// copy a file to the clipboard using Windows PowerShell.
fn copy_file_powershell(file: PathBuf) -> anyhow::Result<()> {
    debug!("Running in WSL environment, using WSL clipboard integration");
    let absolute_path = fs::canonicalize(file)?;
    let absolute_path = absolute_path
        .to_str()
        .ok_or(anyhow!("Path could not be converted to UTF-8"))?;
    debug!(
        "Copying file '{}' to clipboard using PowerShell",
        absolute_path
    );
    duct::cmd!(
        "powershell.exe",
        "-sta",
        "Set-Clipboard",
        "-Path",
        absolute_path
    )
    .run()?;

    Ok(())
}

/// copy a file to the clipboard on macOS using AppleScript.
fn copy_file_osx(relative_path: PathBuf) -> anyhow::Result<()> {
    debug!("Running in OSX environment, using Applescript integration");
    let absolute_path = fs::canonicalize(relative_path)?;
    let absolute_path = absolute_path
        .to_str()
        .ok_or(anyhow!("Path could not be converted to UTF-8"))?;

    debug!(
        "Copying file '{}' to clipboard using Applescript (osascript)",
        absolute_path
    );
    duct::cmd!(
        "osascript",
        "-e",
        format!("set the clipboard to POSIX file \"{absolute_path}\"")
    )
    .run()?;
    Ok(())
}
