mod config;
mod fzf;
mod history;
mod repo;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::Path;
use std::process::ExitCode;

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
        /// Show every project, ignoring the GIT_PROJECTS_TARGETS filter
        scope: Option<Scope>,
    },
}

#[derive(Clone, ValueEnum)]
enum Scope {
    All,
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

    let sorted = history::sort_by_history(filtered);

    // Cancelling the picker is not an error — print nothing and let the
    // calling shell function see an empty result.
    let Some(selected) = fzf::select(&sorted)? else {
        return Ok(());
    };

    history::record(&selected);
    let full_path = Path::new(&ctx.root).join(&selected);
    println!("{}", full_path.display());

    Ok(())
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let ctx = config::AppContext::load()?;

    match cli.command {
        Commands::Choice { scope } => cmd_choice(&ctx, scope.is_some()),
    }
}

fn main() -> ExitCode {
    if let Err(e) = run() {
        eprintln!("Error: {e:#}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
