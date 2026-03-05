use console::Term;
use dialoguer::Select;
use tracing_subscriber::{fmt, EnvFilter};

use super::runner;
use crate::{paths, wizard};

/// Run the interactive menu (shown when no subcommand is given).
pub async fn run_interactive_menu() -> anyhow::Result<()> {
    let term = Term::stderr();
    if !term.is_term() {
        anyhow::bail!(
            "Interactive mode requires a TTY. Use a subcommand instead:\n  \
             cimishi query --config <path>\n  \
             cimishi init"
        );
    }

    println!("\nCimishi — Interactive Mode\n");

    let menu_items = &[
        "Run a saved query config",
        "Compare (coming soon)",
        "Create new config (init)",
        "Exit",
    ];

    let selection = Select::new()
        .with_prompt("What would you like to do?")
        .items(menu_items)
        .default(0)
        .interact()?;

    match selection {
        0 => run_saved_config().await?,
        1 => {
            println!("\nCompare is not yet implemented. Stay tuned!");
        }
        2 => {
            wizard::flow::run_wizard().await?;
        }
        3 => {
            println!("Goodbye!");
        }
        _ => unreachable!(),
    }

    Ok(())
}

async fn run_saved_config() -> anyhow::Result<()> {
    // Check both .cimishi/config/ and the global config directory, merging results
    let local_dir = paths::local_config_dir();
    let global_dir = paths::configs_dir();

    let mut entries = runner::scan_configs(&local_dir);
    let global_entries = runner::scan_configs(&global_dir);

    // Add global entries that don't conflict with local ones (by name)
    for ge in global_entries {
        if !entries.iter().any(|e| e.name == ge.name) {
            entries.push(ge);
        }
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    if entries.is_empty() {
        println!(
            "\nNo config files found in .cimishi/config/ or {}.\n\
             Run `cimishi init` to create one.",
            global_dir.display()
        );
        return Ok(());
    }

    let labels: Vec<String> = entries
        .iter()
        .map(|e| format!("{} ({})", e.name, e.path.display()))
        .collect();

    let idx = Select::new()
        .with_prompt("Select a config to run")
        .items(&labels)
        .default(0)
        .interact()?;

    // Initialize logging before running pipeline
    let filter = EnvFilter::new("info");
    fmt().with_env_filter(filter).with_target(false).init();

    println!("\nRunning: {}\n", entries[idx].path.display());
    runner::run_selected_config(&entries[idx].path).await?;

    Ok(())
}
