use std::path::{Path, PathBuf};
use std::process;

use clap::{Parser, Subcommand};
use log::error;

use fs_cleaner::{analyzer, journal, mover};

#[derive(Parser)]
#[command(
    name = "fs-cleaner",
    version,
    about = "Safely flatten redundant nested directory structures"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Analyze a directory for redundant nesting
    Analyze {
        /// Target directory to analyze
        path: PathBuf,
    },

    /// Apply flattening (moves files up one level)
    Apply {
        /// Target directory to flatten
        path: PathBuf,

        /// Show what would happen without making changes
        #[arg(long)]
        dry_run: bool,
    },

    /// Roll back a previous apply using the journal
    Rollback {
        /// Directory where the journal was saved
        path: PathBuf,
    },

    /// Output a JSON report of detected nesting
    Report {
        /// Target directory to report on
        path: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    let log_level = if cli.verbose { "debug" } else { "info" };
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .format_timestamp(None)
        .init();

    let result = match cli.command {
        Command::Analyze { path } => cmd_analyze(&path),
        Command::Apply { path, dry_run } => cmd_apply(&path, dry_run),
        Command::Rollback { path } => cmd_rollback(&path),
        Command::Report { path } => cmd_report(&path),
    };

    if let Err(e) = result {
        error!("{e}");
        process::exit(1);
    }
}

fn cmd_analyze(path: &Path) -> fs_cleaner::Result<()> {
    let candidates = analyzer::detect_nesting(path)?;

    if candidates.is_empty() {
        println!("No redundant nesting detected in {}", path.display());
        return Ok(());
    }

    for c in &candidates {
        println!("Detected redundant nesting: {}", c.nested.display());
        println!("Proposed moves:");
        for child in &c.children {
            if let Some(name) = child.file_name() {
                let dest = c.parent.join(name);
                if dest == c.nested {
                    continue;
                }
                println!("  {} -> {}", name.to_string_lossy(), dest.display());
            }
        }

        let report = fs_cleaner::scanner::scan(c);
        if report.collisions.is_empty() {
            println!("\nNo collisions detected.");
        } else {
            println!("\nCollisions detected ({}):", report.collisions.len());
            for col in &report.collisions {
                println!(
                    "  {} conflicts with {}",
                    col.source.display(),
                    col.existing.display()
                );
            }
        }

        if report.symlink_risks.is_empty() {
            println!("No symlink risks detected.");
        } else {
            println!("Symlink risks ({}):", report.symlink_risks.len());
            for risk in &report.symlink_risks {
                println!("  {} -> {}", risk.link.display(), risk.target.display());
            }
        }
    }

    println!("\nRun with `apply {}` to execute.", path.display());
    Ok(())
}

fn cmd_apply(path: &Path, dry_run: bool) -> fs_cleaner::Result<()> {
    let candidates = analyzer::detect_nesting(path)?;

    if candidates.is_empty() {
        println!("Nothing to flatten.");
        return Ok(());
    }

    for candidate in &candidates {
        if dry_run {
            println!("[dry-run] Would flatten: {}", candidate.nested.display());
        }

        let result = mover::flatten(candidate, dry_run)?;

        for m in &result.moved {
            let prefix = if dry_run { "[dry-run] " } else { "" };
            println!("{prefix}{} -> {}", m.from.display(), m.to.display());
        }

        if !dry_run {
            let mut j = journal::Journal::new();
            j.record(result.moved);
            let journal_path = j.save(&candidate.parent)?;
            println!("Journal saved to {}", journal_path.display());
        }
    }

    Ok(())
}

fn cmd_rollback(path: &Path) -> fs_cleaner::Result<()> {
    let j = journal::Journal::load(path)?;
    let count = j.rollback()?;
    println!("Rolled back {count} move(s).");
    Ok(())
}

fn cmd_report(path: &Path) -> fs_cleaner::Result<()> {
    let candidates = analyzer::detect_nesting(path)?;

    #[derive(serde::Serialize)]
    struct Report {
        path: PathBuf,
        candidates: Vec<CandidateReport>,
    }

    #[derive(serde::Serialize)]
    struct CandidateReport {
        nested: PathBuf,
        children: Vec<PathBuf>,
        collisions: usize,
        symlink_risks: usize,
    }

    let mut report = Report {
        path: path.to_path_buf(),
        candidates: Vec::new(),
    };

    for c in &candidates {
        let scan = fs_cleaner::scanner::scan(c);
        report.candidates.push(CandidateReport {
            nested: c.nested.clone(),
            children: c.children.clone(),
            collisions: scan.collisions.len(),
            symlink_risks: scan.symlink_risks.len(),
        });
    }

    let json = serde_json::to_string_pretty(&report)
        .map_err(|e| fs_cleaner::Error::Other(e.to_string()))?;
    println!("{json}");
    Ok(())
}
