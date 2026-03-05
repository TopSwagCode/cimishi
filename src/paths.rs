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

/// `<config_dir>/config/`
pub fn configs_dir() -> PathBuf {
    config_dir().join("config")
}

/// `<config_dir>/query/`
pub fn queries_dir() -> PathBuf {
    config_dir().join("query")
}

/// `.cimishi/` — local project directory (like `.github/`)
pub fn local_dir() -> PathBuf {
    PathBuf::from(".cimishi")
}

/// `.cimishi/config/`
pub fn local_config_dir() -> PathBuf {
    local_dir().join("config")
}

/// `.cimishi/query/`
pub fn local_query_dir() -> PathBuf {
    local_dir().join("query")
}

/// `.cimishi/data/`
pub fn local_data_dir() -> PathBuf {
    local_dir().join("data")
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
    println!("Global config directory:  {}", configs_dir().display());
    println!("Global query directory:   {}", queries_dir().display());
    println!("Global data directory:    {}", data_dir().display());
    println!("Local project directory:  {}", local_dir().display());
}
