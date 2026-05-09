pub mod formatters;

use clap::Parser;
use gitinspector_core::{Config, CliGitProvider, GitProvider};
use gitinspector_core::timeline::generate_timeline;
use gitinspector_core::blame::compute_blame;
use gitinspector_core::filtering::Filter;
use std::process::exit;
use std::sync::Arc;
use formatters::Report;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The repository path to inspect
    #[arg(default_value = ".")]
    repo: String,

    /// Output format (text, html, json, xml)
    #[arg(short = 'F', long, default_value = "text")]
    format: String,

    /// Calculate timeline
    #[arg(short = 'T', long)]
    timeline: bool,

    /// Calculate author responsibilities (blame)
    #[arg(short = 'r', long)]
    responsibilities: bool,

    /// Include metrics
    #[arg(short = 'm', long)]
    metrics: bool,

    /// Exclude patterns (author:PATTERN, email:PATTERN, revision:PATTERN, message:PATTERN, file:PATTERN)
    #[arg(short = 'x', long)]
    exclude: Vec<String>,

    /// File types to include (comma separated extensions)
    #[arg(short = 'f', long)]
    file_types: Option<String>,

    /// Enable grading mode (macro for several other flags)
    #[arg(long)]
    grading: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    let mut config = Config::default();
    config.repo_path = args.repo.clone();
    config.timeline = args.timeline || args.grading;
    config.include_metrics = args.metrics || args.grading;
    config.responsibilities = args.responsibilities || args.grading;
    
    if args.grading {
        config.hard = true;
        config.use_weeks = true;
        config.list_file_types = true;
    }

    if let Some(ft) = args.file_types {
        config.extensions = ft.split(',').map(|s| s.trim().to_string()).collect();
    }

    let provider = Arc::new(CliGitProvider);
    let filter = Arc::new(Filter::new(&args.exclude).unwrap_or_default());
    let config_arc = Arc::new(config.clone());

    if args.format == "text" {
        println!("Analyzing repository at: {}", config.repo_path);
    }

    match provider.get_commits(&config).await {
        Ok(raw_output) => {
            let commits = gitinspector_core::analysis::parse_commits(&raw_output, &config, &filter);
            let stats = gitinspector_core::analysis::compute_author_stats(&commits);
            let file_stats = gitinspector_core::analysis::compute_file_stats(&commits);
            
            let mut report = Report {
                repo_name: get_repo_name(&config.repo_path),
                branch_name: get_branch_name(&config.repo_path),
                remote_url: get_remote_url(&config.repo_path),
                authors: stats,
                file_stats,
                timeline: None,
                blame: None,
                metrics_enabled: config.include_metrics,
            };

            if config.timeline {
                let periods = generate_timeline(&commits, config.use_weeks);
                report.timeline = Some(periods);
            }

            if config.responsibilities {
                if args.format == "text" {
                    println!("--- Blame Responsibilities ---");
                    println!("Analyzing files (this may take a moment)...");
                }
                
                if let Ok(files) = provider.get_tracked_files(&config).await {
                    // compute_blame now handles file filtering internally
                    if let Ok(blame_stats) = compute_blame(provider.clone(), config_arc.clone(), filter.clone(), files).await {
                        report.blame = Some(blame_stats);
                    }
                }
            }

            let output = match args.format.to_lowercase().as_str() {
                "json" => formatters::json::format(&report),
                "xml" => formatters::xml::format(&report),
                "html" => formatters::html::format(&report),
                "markdown" | "md" => formatters::markdown::format(&report),
                "text" | _ => formatters::text::format(&report),
            };

            println!("{}", output);
        }
        Err(e) => {
            eprintln!("Error executing git: {}", e);
            exit(1);
        }
    }
}

fn get_repo_name(repo_path: &str) -> String {
    std::path::Path::new(repo_path)
        .canonicalize()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "Unknown Repository".to_string())
}

fn get_branch_name(repo_path: &str) -> String {
    use std::process::Command;
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("branch")
        .arg("--show-current")
        .output()
        .ok();

    if let Some(output) = output {
        if output.status.success() {
            let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !name.is_empty() {
                return name;
            }
        }
    }
    "HEAD".to_string()
}

fn get_remote_url(repo_path: &str) -> Option<String> {
    use std::process::Command;
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_path)
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .output()
        .ok()?;

    if output.status.success() {
        let mut url = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if url.ends_with(".git") {
            url.truncate(url.len() - 4);
        }
        // Handle SSH URLs like git@github.com:user/repo.git
        if url.starts_with("git@") {
            url = url.replace(":", "/").replace("git@", "https://");
        }
        Some(url)
    } else {
        None
    }
}
