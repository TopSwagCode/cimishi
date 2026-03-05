use std::path::PathBuf;

/// Returns `~/.config/cimishi/` on Linux and macOS, `%APPDATA%\cimishi\` on Windows.
///
/// On macOS, `dirs::config_dir()` returns `~/Library/Application Support/` which is
/// non-standard for CLI tools. We override this to use `~/.config/` like gh, docker,
/// kubectl, and starship do.
pub fn config_dir() -> PathBuf {
    let base = if cfg!(target_os = "macos") {
        // Use ~/.config on macOS for CLI-friendliness
        dirs::home_dir().map(|h| h.join(".config"))
    } else {
        dirs::config_dir()
    };

    base.unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
    })
    .join("cimishi")
}

/// `<config_dir>/configs/`
pub fn configs_dir() -> PathBuf {
    config_dir().join("configs")
}

/// `<config_dir>/queries/`
pub fn queries_dir() -> PathBuf {
    config_dir().join("queries")
}

/// Returns the data directory for cimishi.
///
/// On macOS, uses `~/.local/share/cimishi/data/` for consistency with the config
/// override. On other platforms, uses `dirs::data_dir()`.
pub fn data_dir() -> PathBuf {
    let base = if cfg!(target_os = "macos") {
        dirs::home_dir().map(|h| h.join(".local").join("share"))
    } else {
        dirs::data_dir()
    };

    base.unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".local")
            .join("share")
    })
    .join("cimishi")
    .join("data")
}

/// Print all resolved paths to stdout.
pub fn print_paths() {
    println!("Config directory:  {}", configs_dir().display());
    println!("Query directory:   {}", queries_dir().display());
    println!("Data directory:    {}", data_dir().display());
}
