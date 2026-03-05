use std::path::{Path, PathBuf};

use crate::{Pipeline, PipelineConfig};

/// A discovered config file entry.
pub struct ConfigEntry {
    pub name: String,
    pub path: PathBuf,
}

/// Scan a directory for config files (*.toml, *.yaml, *.yml, *.json).
pub fn scan_configs(dir: &Path) -> Vec<ConfigEntry> {
    let mut entries = Vec::new();

    let read_dir = match std::fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return entries,
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default()
            .to_lowercase();

        if matches!(ext.as_str(), "toml" | "yaml" | "yml" | "json") {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")
                .to_string();
            entries.push(ConfigEntry { name, path });
        }
    }

    entries.sort_by(|a, b| a.name.cmp(&b.name));
    entries
}

/// Load and run a config file through the pipeline.
pub async fn run_selected_config(path: &Path) -> anyhow::Result<()> {
    let config = PipelineConfig::from_file(path)?;
    let pipeline = Pipeline::new(config);
    pipeline.run().await?;
    Ok(())
}
