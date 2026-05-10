use clap::Parser;
use std::process::exit;
use std::sync::Arc;
use std::time::Instant;
use indicatif::{ProgressBar, ProgressStyle};
use console::{style, Emoji};
use tokio::process::Command;
use futures::StreamExt;

mod formatters;
use formatters::Report;

use gitinspector_core::config::Config;
use gitinspector_core::provider::{CliGitProvider, GitProvider};
use gitinspector_core::filtering::Filter;
use gitinspector_core::analysis::{RepoHealth, BranchInfo, FileStats, CommitParser, IncrementalAggregator};

static LOOKING_GLASS: Emoji<'_, '_> = Emoji("🔍  ", "");
static SPARKLE: Emoji<'_, '_> = Emoji("✨  ", "");
static TRUCK: Emoji<'_, '_> = Emoji("🚚  ", "");

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(default_value = ".")]
    repo_path: String,
    #[arg(short = 'F', long, default_value = "text")]
    format: String,
    #[arg(short = 'f', long)]
    file_types: Option<String>,
    #[arg(long)]
    grading: bool,
    #[arg(short = 'T', long)]
    timeline: bool,
    #[arg(short = 'r', long)]
    responsibilities: bool,
    #[arg(short = 'm', long)]
    metrics: bool,
    #[arg(short = 'x', long)]
    exclude: Vec<String>,
}

#[tokio::main]
async fn main() {
    let start_time = Instant::now();
    let args = Args::parse();

    let mut config = Config::default();
    config.repo_path = args.repo_path.clone();
    
    if args.grading {
        config.timeline = true;
        config.responsibilities = true;
        config.include_metrics = true;
    } else {
        config.timeline = args.timeline;
        config.responsibilities = args.responsibilities;
        config.include_metrics = args.metrics;
    }

    if let Some(types) = args.file_types {
        config.extensions = types.split(',').map(|s| s.trim().to_string()).collect();
    }

    let filter_val = match Filter::new(&args.exclude) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("{} Invalid exclusion pattern: {}", style("ERROR").red().bold(), e);
            exit(1);
        }
    };

    let provider: Arc<dyn GitProvider + Send + Sync> = Arc::new(CliGitProvider::new());
    let config = Arc::new(config);
    let filter = Arc::new(filter_val);

    eprintln!("{} {}Analyzing repository at {}...", style("[1/3]").bold().dim(), LOOKING_GLASS, style(&config.repo_path).cyan());

    let pb = ProgressBar::new_spinner();
    pb.set_style(ProgressStyle::default_spinner()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈")
        .template("{spinner:.green} {msg}")
        .unwrap());
    pb.set_message("Initializing git log stream...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    match provider.get_commits(&config).await {
        Ok(mut line_stream) => {
            let mut parser = CommitParser::new(&config, &filter);
            let mut aggregator = IncrementalAggregator::new();
            let mut commit_count = 0;
            let mut pr_count = 0;

            while let Some(line_res) = line_stream.next().await {
                if let Ok(line) = line_res {
                    if let Some(commit) = parser.parse_line(&line) {
                        if commit.subject.contains("Merge pull request #") {
                            pr_count += 1;
                        }
                        aggregator.add_commit(&commit);
                        commit_count += 1;
                        if commit_count % 100 == 0 {
                            pb.set_message(format!("Processed {} commits...", commit_count));
                        }
                    }
                }
            }
            if let Some(commit) = parser.finalize() {
                aggregator.add_commit(&commit);
                commit_count += 1;
            }

            pb.set_message("Finalizing statistics...");
            let (stats, mut file_stats) = aggregator.finalize();
            
            pb.set_message("Auditing file metrics and health...");
            let (mut health, _) = tokio::join!(
                compute_repo_health(config.clone()),
                populate_file_metrics(&config.repo_path, &mut file_stats)
            );
            health.estimated_prs_count = pr_count;

            let mut report = Report {
                repo_name: get_repo_name(&config.repo_path),
                branch_name: get_branch_name(&config.repo_path),
                remote_url: get_remote_url(&config.repo_path),
                authors: stats,
                file_stats,
                timeline: None,
                blame: None,
                health: Some(health),
                metrics_enabled: config.include_metrics,
                version: env!("CARGO_PKG_VERSION").to_string(),
                duration: format!("{:.2}s", start_time.elapsed().as_secs_f32()),
            };

            pb.finish_and_clear();

            if config.responsibilities {
                eprintln!("{} {}Computing code ownership (git blame)...", style("[3/3]").bold().dim(), TRUCK);
                let files = provider.get_tracked_files(&config).await.unwrap_or_default();
                
                let blame_pb = ProgressBar::new(files.len() as u64);
                blame_pb.set_style(ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} files ({eta})")
                    .unwrap()
                    .progress_chars("#>-"));
                
                match gitinspector_core::blame::compute_blame(provider.clone(), config.clone(), filter.clone(), files).await {
                    Ok(blame_stats) => {
                        report.blame = Some(blame_stats);
                        blame_pb.finish_with_message("Blame analysis complete");
                    }
                    Err(e) => {
                        blame_pb.abandon();
                        eprintln!("Error computing blame: {}", e);
                    }
                }
            }

            eprintln!("{} {}Generating {} report...", style("DONE").green().bold(), SPARKLE, style(&args.format).yellow());

            let output = match args.format.to_lowercase().as_str() {
                "json" => formatters::json::format(&report),
                "xml" => formatters::xml::format(&report),
                "html" => formatters::html::format(&report),
                "markdown" | "md" => formatters::markdown::format(&report),
                "text" | _ => formatters::text::format(&report),
            };

            println!("{}", output);
            let duration = start_time.elapsed();
            eprintln!("\n{} Analysis complete in {:.2}s", style("FINISH").green().bold(), duration.as_secs_f32());
            eprintln!("Processed {} commits for {} authors.", commit_count, report.authors.len());
        }
        Err(e) => {
            pb.abandon();
            eprintln!("{} Error executing git: {}", style("ERROR").red().bold(), e);
            exit(1);
        }
    }
}

async fn populate_file_metrics(repo_path: &str, file_stats: &mut [FileStats]) {
    let size_output = Command::new("git").arg("-C").arg(repo_path).arg("ls-tree").arg("-r").arg("-l").arg("HEAD").output().await;
    if let Ok(output) = size_output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let size = parts[3].parse::<u32>().unwrap_or(0);
                let path = parts[4..].join(" ");
                if let Some(stat) = file_stats.iter_mut().find(|s| s.path == path) {
                    stat.total_lines = size;
                }
            }
        }
    }
    let loc_output = Command::new("git").arg("-C").arg(repo_path).arg("grep").arg("-c").arg("^").arg("HEAD").output().await;
    if let Ok(output) = loc_output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 3 {
                let path = parts[1];
                let count = parts[2].parse::<u32>().unwrap_or(0);
                if let Some(stat) = file_stats.iter_mut().find(|s| s.path == path) {
                    stat.loc = count;
                }
            }
        }
    }
}

async fn compute_repo_health(config: Arc<Config>) -> RepoHealth {
    let mut health = RepoHealth::default();
    let branch_output = Command::new("git").arg("-C").arg(&config.repo_path).arg("for-each-ref").arg("--format=%(refname:short)|%(committerdate:unix)|%(authorname)").arg("refs/heads").arg("refs/remotes/origin").output().await;
    if let Ok(output) = branch_output {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let now = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
        let ninety_days = 90 * 24 * 3600;
        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 3 {
                let name = parts[0].to_string();
                let ts = parts[1].parse::<u64>().unwrap_or(0);
                let author = parts[2].to_string();
                let is_stale = (now - ts) > ninety_days;
                if is_stale { health.stale_branches_count += 1; }
                let last_date = chrono::TimeZone::timestamp_opt(&chrono::Utc, ts as i64, 0).single().map(|dt| dt.format("%b %d, %Y").to_string()).unwrap_or_else(|| "Unknown".to_string());
                health.active_branches.push(BranchInfo { name, last_commit_date: last_date, last_author: author, is_stale });
            }
        }
    }
    let output = Command::new("git").arg("-C").arg(&config.repo_path).arg("ls-tree").arg("-r").arg("-l").arg("HEAD").output().await;
    if let Ok(output) = output {
        let mut files: Vec<FileStats> = String::from_utf8_lossy(&output.stdout).lines().filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                let size = parts[3].parse::<u32>().unwrap_or(0);
                let path = parts[4..].join(" ");
                Some(FileStats { path, total_lines: size, ..Default::default() })
            } else { None }
        }).collect();
        files.sort_by(|a, b| b.total_lines.cmp(&a.total_lines));
        files.truncate(10);
        health.large_files = files;
    }
    health
}

fn get_repo_name(repo_path: &str) -> String {
    let path = std::path::Path::new(repo_path);
    if let Ok(abs_path) = path.canonicalize() {
        if let Some(name) = abs_path.file_name() {
            return name.to_string_lossy().to_string();
        }
    }
    path.file_name().map(|n| n.to_string_lossy().to_string()).filter(|n| n != "." && n != "..").unwrap_or_else(|| "Repository".to_string())
}

fn get_branch_name(repo_path: &str) -> String {
    let output = std::process::Command::new("git").arg("-C").arg(repo_path).arg("branch").arg("--show-current").output();
    if let Ok(output) = output {
        if output.status.success() {
            let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !name.is_empty() { return name; }
        }
    }
    "HEAD".to_string()
}

fn get_remote_url(repo_path: &str) -> Option<String> {
    let output = std::process::Command::new("git").arg("-C").arg(repo_path).arg("remote").arg("get-url").arg("origin").output();
    if let Ok(output) = output {
        if output.status.success() {
            let mut url = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if url.ends_with(".git") { url.truncate(url.len() - 4); }
            if url.starts_with("git@") { url = url.replace(":", "/").replace("git@", "https://"); }
            return Some(url);
        }
    }
    None
}
