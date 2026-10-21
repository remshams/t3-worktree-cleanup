mod apply;
mod cli;
mod git;
mod plan;
mod store;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Command, SelectionArgs};
use plan::{Plan, Selection};
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode> {
    match cli.command {
        Command::Plan(args) => {
            print(&build_plan(args)?.without_recent())?;
            Ok(ExitCode::SUCCESS)
        }
        Command::Apply {
            selection,
            force,
            paths,
        } => {
            let outcomes = apply::apply(&build_plan(selection)?, &paths, force);
            print(&outcomes)?;
            let failed = outcomes
                .iter()
                .any(|o| o.status == apply::OutcomeStatus::Failed);
            Ok(if failed {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            })
        }
    }
}

fn build_plan(args: SelectionArgs) -> Result<Plan> {
    let base_dir = match args.base_dir {
        Some(dir) => dir,
        None => default_base_dir()?,
    };
    let threads = store::read_threads(&base_dir)?;
    let selection = Selection {
        older_than: args.older_than,
        projects: args.projects,
    };
    Ok(Plan::build(threads, &selection, chrono::Utc::now()))
}

fn default_base_dir() -> Result<PathBuf> {
    if let Some(home) = std::env::var_os("T3CODE_HOME") {
        return Ok(PathBuf::from(home));
    }
    let home = std::env::var_os("HOME").context("HOME is not set; pass --base-dir")?;
    Ok(PathBuf::from(home).join(".t3"))
}

fn print(value: &impl serde::Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
