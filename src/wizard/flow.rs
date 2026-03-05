use std::fs;
use std::path::Path;

use console::Term;
use dialoguer::{Input, MultiSelect, Select};

use super::templates;
use crate::paths;

/// Run the interactive init wizard. Returns Ok(()) on success or user cancellation.
pub async fn run_wizard() -> anyhow::Result<()> {
    let term = Term::stderr();
    if !term.is_term() {
        anyhow::bail!(
            "Interactive wizard requires a TTY. Use --config to pass a config file directly."
        );
    }

    println!("\nCimishi — Config Wizard\n");

    // Step 1: Config type
    let config_types = &["download example (quick start)", "query", "compare"];
    let config_type_idx = Select::new()
        .with_prompt("What would you like to do?")
        .items(config_types)
        .default(0)
        .interact()?;

    if config_type_idx == 0 {
        return super::example::download_example().await;
    }

    if config_types[config_type_idx] == "compare" {
        println!("\nCompare configs are not yet implemented. Stay tuned!");
        return Ok(());
    }

    // Step 2: Name
    let name: String = Input::new()
        .with_prompt("Config name (alphanumeric, hyphens, underscores)")
        .validate_with(|input: &String| -> Result<(), String> {
            if input.is_empty() {
                return Err("Name cannot be empty".into());
            }
            if !input
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
            {
                return Err(
                    "Only alphanumeric characters, hyphens, and underscores are allowed".into(),
                );
            }
            Ok(())
        })
        .interact_text()?;

    // Step 3: Source type
    let source_types = &["local", "s3", "azure", "gcs"];
    let source_idx = Select::new()
        .with_prompt("Source type")
        .items(source_types)
        .default(0)
        .interact()?;
    let source_type = source_types[source_idx];

    // Step 4: Query type
    let query_types = &["file", "inline"];
    let query_idx = Select::new()
        .with_prompt("Query type")
        .items(query_types)
        .default(0)
        .interact()?;
    let query_type = query_types[query_idx];

    // Step 5: Output formats
    let format_options = &["csv", "json", "terminal"];
    let format_indices = MultiSelect::new()
        .with_prompt("Output formats (space to toggle, enter to confirm)")
        .items(format_options)
        .defaults(&[true, false, false])
        .interact()?;

    if format_indices.is_empty() {
        anyhow::bail!("At least one output format must be selected");
    }

    let selected_formats: Vec<String> = format_indices
        .iter()
        .map(|&i| format_options[i].to_string())
        .collect();

    // Step 6: Write files
    let config_path = paths::configs_dir().join(format!("{}.toml", name));
    let query_path = paths::queries_dir().join(format!("{}.sparql", name));

    // Check for conflicts
    let mut files_to_write: Vec<(&Path, String)> = Vec::new();

    let config_content = templates::config_toml(&name, source_type, query_type, &selected_formats);
    if config_path.exists() {
        let overwrite = Select::new()
            .with_prompt(format!(
                "{} already exists. Overwrite?",
                config_path.display()
            ))
            .items(&["yes", "no"])
            .default(1)
            .interact()?;
        if overwrite == 1 {
            println!("Skipping config file.");
        } else {
            files_to_write.push((config_path.as_path(), config_content.clone()));
        }
    } else {
        files_to_write.push((config_path.as_path(), config_content.clone()));
    }

    if query_type == "file" {
        let sparql_content = templates::sparql_query().to_string();
        if query_path.exists() {
            let overwrite = Select::new()
                .with_prompt(format!(
                    "{} already exists. Overwrite?",
                    query_path.display()
                ))
                .items(&["yes", "no"])
                .default(1)
                .interact()?;
            if overwrite == 1 {
                println!("Skipping query file.");
            } else {
                files_to_write.push((query_path.as_path(), sparql_content));
            }
        } else {
            files_to_write.push((query_path.as_path(), sparql_content));
        }
    }

    // Write files
    for (path, content) in &files_to_write {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
    }

    // Step 7: Summary
    println!("\n--- Files created ---");
    for (path, _) in &files_to_write {
        println!("  {}", path.display());
    }

    println!("\n--- Next steps ---");
    println!(
        "  1. Edit the config:  {}",
        paths::configs_dir()
            .join(format!("{}.toml", name))
            .display()
    );
    if query_type == "file" {
        println!(
            "  2. Edit the query:   {}",
            paths::queries_dir()
                .join(format!("{}.sparql", name))
                .display()
        );
    }
    println!(
        "  3. Run the pipeline: cimishi query --config {}",
        paths::configs_dir()
            .join(format!("{}.toml", name))
            .display()
    );

    match source_type {
        "s3" => {
            println!("\n  S3 auth: export AWS_ACCESS_KEY_ID=... AWS_SECRET_ACCESS_KEY=...");
        }
        "azure" => {
            println!("\n  Azure auth: export AZURE_STORAGE_ACCOUNT_KEY=...");
        }
        "gcs" => {
            println!("\n  GCS auth: export GOOGLE_APPLICATION_CREDENTIALS=/path/to/key.json");
        }
        _ => {}
    }

    println!();
    Ok(())
}
