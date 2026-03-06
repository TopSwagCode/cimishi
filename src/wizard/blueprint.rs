use std::path::PathBuf;

use serde::Deserialize;

use crate::paths;

#[derive(Debug, Deserialize)]
pub struct BlueprintConfig {
    pub blueprint: BlueprintMetadata,
    #[serde(default)]
    pub configs: Vec<BlueprintFile>,
    #[serde(default)]
    pub queries: Vec<BlueprintFile>,
    #[serde(default)]
    pub data: Vec<BlueprintFile>,
}

#[derive(Debug, Deserialize)]
pub struct BlueprintMetadata {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BlueprintFile {
    pub url: String,
    pub filename: Option<String>,
}

impl BlueprintFile {
    fn resolved_filename(&self) -> String {
        if let Some(ref name) = self.filename {
            return name.clone();
        }
        self.url
            .rsplit('/')
            .next()
            .unwrap_or("download")
            .to_string()
    }
}

/// Load a blueprint from a local file path or a URL.
pub async fn load_blueprint(source: &str) -> anyhow::Result<BlueprintConfig> {
    let content = if source.starts_with("http://") || source.starts_with("https://") {
        let client = reqwest::Client::new();
        let resp = client.get(source).send().await?;
        if !resp.status().is_success() {
            anyhow::bail!("Failed to fetch blueprint: HTTP {}", resp.status());
        }
        resp.text().await?
    } else {
        std::fs::read_to_string(source)?
    };

    let config: BlueprintConfig = if source.ends_with(".json") {
        serde_json::from_str(&content)?
    } else if source.ends_with(".yaml") || source.ends_with(".yml") {
        serde_yaml::from_str(&content)?
    } else {
        toml::from_str(&content)?
    };

    Ok(config)
}

/// Download all files listed in a blueprint to the appropriate `.cimishi/` directories.
pub async fn download_blueprint(config: BlueprintConfig) -> anyhow::Result<()> {
    println!(
        "\nInstalling blueprint: {}",
        config.blueprint.name
    );
    if let Some(ref desc) = config.blueprint.description {
        println!("  {}", desc);
    }
    println!();

    let client = reqwest::Client::new();

    let categories: &[(&str, &[BlueprintFile], PathBuf)] = &[
        ("configs", &config.configs, paths::local_config_dir()),
        ("queries", &config.queries, paths::local_query_dir()),
        ("data", &config.data, paths::local_data_dir()),
    ];

    let mut total = 0usize;

    for &(label, files, ref dir) in categories {
        if files.is_empty() {
            continue;
        }
        println!("[{}]", label);
        for file in files {
            let dest = dir.join(file.resolved_filename());
            super::example::download_file(&client, &file.url, &dest).await?;
            total += 1;
        }
        println!();
    }

    if total == 0 {
        println!("Blueprint contains no files to download.");
    } else {
        println!("Done — {} file(s) installed.", total);
    }

    if !config.configs.is_empty() {
        let config_dir = paths::local_config_dir();
        println!("\n--- Run it ---");
        for file in &config.configs {
            let name = file.resolved_filename();
            println!("  cimishi query --config {}", config_dir.join(&name).display());
        }
        println!();
    }

    Ok(())
}
