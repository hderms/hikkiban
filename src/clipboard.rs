use std::{fs, io, path::PathBuf};

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
    Osx,
    WSL2,
    Generic,
    XWindows,
}

/// Detect the current operating system or runtime environment.
pub(crate) fn get_os_env_target(allow_wsl: bool) -> OSEnvTarget {
    let info = os_info::get();
    if is_wsl::is_wsl() && allow_wsl {
        OSEnvTarget::WSL2
    } else {
        if info.os_type() == os_info::Type::Macos {
            OSEnvTarget::Osx
        } else if which::which("xclip").is_ok() {
            OSEnvTarget::XWindows
        } else {
            OSEnvTarget::Generic
        }
    }
}

/// Platform-specific implementation of CopyFileClipboard for OSEnvTarget.
impl CopyFileClipboard for OSEnvTarget {
    fn copy_file(&self, filepath: PathBuf) -> anyhow::Result<()> {
        let absolute_path = fs::canonicalize(&filepath)?;

        match self {
            OSEnvTarget::Osx => copy_file_osx(absolute_path),
            OSEnvTarget::WSL2 => copy_file_powershell(absolute_path),
            OSEnvTarget::XWindows => copy_file_xclip(absolute_path),
            OSEnvTarget::Generic => {
                let content = std::fs::read_to_string(absolute_path)?;
                clipboard_anywhere::set_clipboard(&content)?;
                Ok(())
            }
        }
    }
}

/// copy a file to the clipboard using Windows PowerShell.
fn copy_file_powershell(absolute_path: PathBuf) -> anyhow::Result<()> {
    debug!("Running in WSL environment, using WSL clipboard integration");
    let absolute_path = absolute_path
        .to_str()
        .ok_or(anyhow!("Path could not be converted to UTF-8"))?;
    debug!(
        "Copying file '{}' to clipboard using PowerShell",
        absolute_path
    );

    /// Helper function to run PowerShell command for copying file to clipboard.
    fn run_powershell_command(path: &str) -> io::Result<()> {
        duct::cmd!("powershell.exe", "-sta", "Set-Clipboard", "-Path", path)
            .stderr_null()
            .run()
            .map(|_| ())
    }

    //Produces a path like \\wsl.localhost\Ubuntu-Foobar\home\user\file.txt from a linux path like /home/user/file.txt
    let wsl2_relative_path = duct::cmd!("wslpath", "-w", absolute_path)
        .read()?
        .trim()
        .to_string();

    run_powershell_command(absolute_path)
        .or_else(|_| run_powershell_command(&wsl2_relative_path))?;

    Ok(())
}

/// copy a file to the clipboard on macOS using AppleScript.
fn copy_file_osx(absolute_path: PathBuf) -> anyhow::Result<()> {
    debug!("Running in OSX environment, using Applescript integration");
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

/// copy a file to the clipboard on *nix systems with xclip installed.
fn copy_file_xclip(absolute_path: PathBuf) -> anyhow::Result<()> {
    debug!("Running in nix environment with xclip, using xclip integration");

    let absolute_path = absolute_path
        .to_str()
        .ok_or(anyhow!("Path could not be converted to UTF-8"))?;

    debug!("Copying file '{}' to clipboard using xclip ", absolute_path);
    duct::cmd!("xclip", "-sel", "clip", "-in", absolute_path).run()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_os_env_target_should_not_fail() {
        let target = get_os_env_target(true);
        match target {
            OSEnvTarget::OSX => println!("Detected OSX environment"),
            OSEnvTarget::WSL2 => println!("Detected WSL2 environment"),
            OSEnvTarget::Generic => println!("Detected Generic environment"),
            OSEnvTarget::XWindows => println!("Detected XWindows environment"),
        }
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn test_copy_file_osx() {
        use std::{fs::File, io::Read};

        let cargo_path = std::env::current_dir().unwrap().join("Cargo.toml");
        copy_file_osx(cargo_path.clone()).unwrap();
        let path = duct::cmd!(
            "osascript",
            "-e",
            "POSIX path of (the clipboard as «class furl»)"
        )
        .read()
        .unwrap();
        let path = path.trim();
        assert_eq!(cargo_path.to_str().unwrap(), path);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_copy_file_wsl() {
        let cargo_path = std::env::current_dir().unwrap().join("Cargo.toml");

        //copy the file using powershell into the clipboard
        copy_file_powershell(cargo_path.clone()).unwrap();

        //Get the first file path from the clipboard back using powershell and expand the path to a full windows path
        let path = duct::cmd!(
            "powershell.exe",
            "-sta",
            "(Get-Clipboard -Format FileDropList) | Select-Object -First 1 -ExpandProperty FullName"
        )
        .read()
        .unwrap();

        //convert back to Linux path for comparison
        let path = duct::cmd!("wslpath", "-u", path).read().unwrap();
        let path = path.trim();
        assert_eq!(cargo_path.to_str().unwrap(), path);
    }
}
