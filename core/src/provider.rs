use tokio::process::Command;
use crate::config::Config;

#[async_trait::async_trait]
pub trait GitProvider {
    async fn get_commits(&self, config: &Config) -> Result<String, String>;
    async fn get_tracked_files(&self, config: &Config) -> Result<Vec<String>, String>;
    async fn get_blame_file(&self, config: &Config, file_path: &str) -> Result<String, String>;
    async fn get_branches(&self, config: &Config) -> Result<Vec<String>, String>;
}

pub struct CliGitProvider;

impl CliGitProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl GitProvider for CliGitProvider {
    async fn get_commits(&self, config: &Config) -> Result<String, String> {
        let mut cmd = Command::new("git");
        cmd.current_dir(&config.repo_path);
        cmd.arg("log");
        cmd.arg("--numstat");
        cmd.arg("--pretty=format:commit|%h|%an|%ae|%ad|%s");
        cmd.arg("--no-merges");
        cmd.arg("--date=unix");

        if let Some(since) = &config.since {
            cmd.arg(format!("--since={}", since));
        }
        if let Some(until) = &config.until {
            cmd.arg(format!("--until={}", until));
        }

        let output = cmd.output().await.map_err(|e| e.to_string())?;

        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn get_tracked_files(&self, config: &Config) -> Result<Vec<String>, String> {
        let mut cmd = Command::new("git");
        cmd.current_dir(&config.repo_path);
        cmd.arg("ls-tree");
        cmd.arg("-r");
        cmd.arg("HEAD");
        cmd.arg("--name-only");

        let output = cmd.output().await.map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        let files = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.to_string())
            .collect();
        Ok(files)
    }

    async fn get_blame_file(&self, config: &Config, file_path: &str) -> Result<String, String> {
        let mut cmd = Command::new("git");
        cmd.current_dir(&config.repo_path);
        cmd.arg("blame");
        cmd.arg("-w");
        cmd.arg("-M");
        cmd.arg("-C");
        cmd.arg("--line-porcelain");
        cmd.arg(file_path);

        let output = cmd.output().await.map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    async fn get_branches(&self, config: &Config) -> Result<Vec<String>, String> {
        let mut cmd = Command::new("git");
        cmd.current_dir(&config.repo_path);
        cmd.arg("branch");
        cmd.arg("-a");
        cmd.arg("--no-color");

        let output = cmd.output().await.map_err(|e| e.to_string())?;
        if !output.status.success() {
            return Err(String::from_utf8_lossy(&output.stderr).to_string());
        }

        let branches = String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|s| s.trim().trim_start_matches('*').trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        Ok(branches)
    }
}
