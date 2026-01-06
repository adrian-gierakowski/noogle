use clap::{Parser, Subcommand};
use std::path::PathBuf;
use anyhow::Result;
use directories::ProjectDirs;

mod model;
mod loader;
mod search;
mod tui;
#[cfg(test)]
mod tests;

#[derive(Parser)]
#[command(name = "noogle")]
#[command(about = "Search Nix documentation", long_about = None)]
struct Cli {
    #[arg(short, long, env = "NOOGLE_DATA")]
    data: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,

    /// Search query (if no subcommand is provided, defaults to interactive or oneshot search depending on tty)
    query: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Search interactively or print results
    Search {
        query: String,
        #[arg(long)]
        json: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // 1. Locate data.json
    let data_path = if let Some(p) = cli.data {
        p
    } else {
        // Try default locations
        // 1. Current directory
        let p = PathBuf::from("data.json");
        if p.exists() {
            p
        } else {
             // 2. Default XDG location?
             let dirs = ProjectDirs::from("dev", "noogle", "cli");
             if let Some(dirs) = dirs {
                 let p = dirs.data_dir().join("data.json");
                 if p.exists() {
                     p
                 } else {
                     // 3. Fallback relative to executable?
                     // For now, let's just error if not found or default to "data.json"
                     PathBuf::from("data.json")
                 }
             } else {
                 PathBuf::from("data.json")
             }
        }
    };

    if !data_path.exists() {
        eprintln!("Error: data.json not found at {:?}. Please provide it with --data or place it in current directory.", data_path);
        std::process::exit(1);
    }

    // 2. Load data
    let data = loader::load_data(&data_path)?;

    // 3. Handle commands
    if let Some(command) = cli.command {
        match command {
            Commands::Search { query, json } => {
                let results = search::search(&query, &data);
                if json {
                    // TODO: Output JSON
                    // We need a serializable struct for results
                    #[derive(serde::Serialize)]
                    struct JsonRes<'a> {
                        item: &'a model::DocItem,
                        score: i64,
                    }
                    let json_results: Vec<JsonRes> = results.iter().map(|r| JsonRes { item: r.doc, score: r.score }).collect();
                    println!("{}", serde_json::to_string_pretty(&json_results)?);
                } else {
                    for res in results {
                        println!("{} (score: {})", res.doc.title(), res.score);
                    }
                }
            }
        }
    } else if let Some(query) = cli.query {
         // One shot search
         let results = search::search(&query, &data);
         for res in results {
            println!("{} (score: {})", res.doc.title(), res.score);
         }
    } else {
        // Interactive TUI
        // anyhow::Result expects Box<dyn std::error::Error + Send + Sync + 'static> for errors usually?
        // tui::run_tui returns Result<(), Box<dyn Error>>. Box<dyn Error> might not be Send/Sync.
        // We can map error to anyhow
        tui::run_tui(&data).map_err(|e| anyhow::anyhow!(e.to_string()))?;
    }

    Ok(())
}
