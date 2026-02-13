mod config;
mod fzf;
mod repo;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::Path;

#[derive(Parser)]
#[command(name = "git-projects", version, about = "git projects")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Choose project from list
    #[command(alias = "c")]
    Choice {
        /// Show all projects (ignore targets filter)
        all: Option<String>,
    },
}

fn cmd_choice(ctx: &config::AppContext, show_all: bool) -> Result<()> {
    let repos = repo::list_repos(&ctx.root);

    let filtered: Vec<String> = if show_all || ctx.targets.is_empty() {
        repos
    } else {
        repos
            .into_iter()
            .filter(|r| ctx.targets.iter().any(|t| r.contains(t)))
            .collect()
    };

    let selected = fzf::select(&filtered)?;
    let full_path = Path::new(&ctx.root).join(&selected);
    println!("{}", full_path.display());

    Ok(())
}

fn main() {
    let cli = Cli::parse();
    let ctx = match config::AppContext::load() {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    let result = match cli.command {
        Commands::Choice { all } => cmd_choice(&ctx, all.is_some()),
    };

    if let Err(e) = result {
        // fzf cancelled — exit silently
        let _ = e;
    }
}
