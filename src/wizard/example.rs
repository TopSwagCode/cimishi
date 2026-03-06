use std::fs;
use std::path::PathBuf;

use crate::paths;

const BASE_URL: &str = "https://raw.githubusercontent.com/TopSwagCode/cimishi/refs/heads/master";

struct ExampleFile {
    url: &'static str,
    filename: &'static str,
}

const EXAMPLE_DATA_FILES: &[ExampleFile] = &[ExampleFile {
    url: "examples/data/SampleData.zip",
    filename: "example-data.zip",
}];

const EXAMPLE_QUERY_FILES: &[ExampleFile] = &[ExampleFile {
    url: "examples/queries/query.sparql",
    filename: "example-query.sparql",
}];

fn normalize_path(p: &str) -> String {
    p.replace('\\', "/")
}

fn pipeline_config_json(data_dir: &str, query_path: &str) -> String {
    let data_dir = normalize_path(data_dir);
    let query_path = normalize_path(query_path);
    format!(
        r#"{{
  "pipeline": {{
    "name": "example-pipeline",
    "parallel": true,
    "max_concurrent": 10
  }},
  "sources": [
    {{
      "type": "local",
      "path": "{}",
      "patterns": ["*.xml", "*.rdf", "*.zip", "*.gz"],
      "recursive": true
    }}
  ],
  "processors": [
    {{
      "type": "unzip",
      "archive_patterns": ["*.zip"],
      "gzip_patterns": ["*.gz", "*.gzip"],
      "patterns": ["*.xml", "*.rdf"]
    }},
    {{
      "type": "filter",
      "exclude": ["._*", "__MACOSX/*"]
    }}
  ],
  "query": {{
    "file": "{}",
    "base_iri": "http://example.org/"
  }},
  "output": {{
    "dir": "./.cimishi/output",
    "formats": ["csv", "json", "terminal"],
    "metadata": true,
    "prefix": "example-results"
  }}
}}"#,
        data_dir, query_path
    )
}

/// Download example files and print instructions to get started.
pub async fn download_example() -> anyhow::Result<()> {
    println!("\nDownloading example files...\n");

    let data_dir = paths::local_data_dir();
    let queries_dir = paths::local_query_dir();
    let configs_dir = paths::local_config_dir();

    let client = reqwest::Client::new();

    // Download data files
    for file in EXAMPLE_DATA_FILES {
        let url = format!("{}/{}", BASE_URL, file.url);
        let dest = data_dir.join(file.filename);
        download_file(&client, &url, &dest).await?;
    }

    // Download query files
    for file in EXAMPLE_QUERY_FILES {
        let url = format!("{}/{}", BASE_URL, file.url);
        let dest = queries_dir.join(file.filename);
        download_file(&client, &url, &dest).await?;
    }

    // Write the pipeline config with paths adjusted for the download layout
    fs::create_dir_all(&configs_dir)?;
    let config_path = configs_dir.join("example-pipeline.json");
    let query_path = queries_dir.join("example-query.sparql");
    let config_content =
        pipeline_config_json(&data_dir.to_string_lossy(), &query_path.to_string_lossy());
    fs::write(&config_path, &config_content)?;
    println!("  {} ... OK (generated)", config_path.display());

    println!("\n--- Example ready ---");
    println!(
        "  {}   Sample RDF data (ZIP archive)",
        data_dir.join("example-data.zip").display()
    );
    println!("  {}   SPARQL query", query_path.display());
    println!(
        "  {}   Pipeline config (unzips + queries)",
        config_path.display()
    );

    println!("\n--- Run it ---");
    println!("  cimishi query --config {}", config_path.display());

    println!("\n--- Results ---");
    println!("  Output will be written to ./.cimishi/output/");
    println!("  Look for example-results*.csv and example-results*.metadata.json\n");

    Ok(())
}

pub(super) async fn download_file(
    client: &reqwest::Client,
    url: &str,
    dest: &PathBuf,
) -> anyhow::Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }

    print!("  {} ... ", dest.display());
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        println!("FAILED (HTTP {})", response.status());
        anyhow::bail!("Failed to download {}: HTTP {}", url, response.status());
    }

    let bytes = response.bytes().await?;
    fs::write(dest, &bytes)?;
    println!("OK ({} bytes)", bytes.len());
    Ok(())
}
