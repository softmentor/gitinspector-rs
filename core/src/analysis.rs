use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::config::Config;
use crate::filtering::{Filter, FilterType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub insertions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub hash: String,
    pub author_name: String,
    pub author_email: String,
    pub date: String,
    pub subject: String,
    pub insertions: u32,
    pub deletions: u32,
    pub changes: Vec<FileChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthorStats {
    pub name: String,
    pub email: String,
    pub commits: u32,
    pub insertions: u32,
    pub deletions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileStats {
    pub path: String,
    pub commits: u32,
    pub insertions: u32,
    pub deletions: u32,
}

pub fn parse_commits(raw_output: &str, config: &Config, filter: &Filter) -> Vec<Commit> {
    let mut commits = Vec::new();
    let mut current_commit: Option<Commit> = None;
    let mut commit_touches_valid_file = false;

    for line in raw_output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("commit|") {
            if let Some(commit) = current_commit.take() {
                if commit_touches_valid_file {
                    commits.push(commit);
                }
            }
            
            let parts: Vec<&str> = line.splitn(6, '|').collect();
            if parts.len() == 6 {
                let hash = parts[1].to_string();
                let author_name = parts[2].to_string();
                let author_email = parts[3].to_string();
                let date = parts[4].to_string();
                let subject = parts[5].to_string();

                // Advanced Filtering
                if filter.is_excluded(&hash, FilterType::Revision) ||
                   filter.is_excluded(&author_name, FilterType::Author) ||
                   filter.is_excluded(&author_email, FilterType::Email) ||
                   filter.is_excluded(&subject, FilterType::Message) {
                    current_commit = None;
                    commit_touches_valid_file = false;
                    continue;
                }

                current_commit = Some(Commit {
                    hash,
                    author_name,
                    author_email,
                    date,
                    subject,
                    insertions: 0,
                    deletions: 0,
                    changes: Vec::new(),
                });
                commit_touches_valid_file = false;
            }
        } else if let Some(ref mut commit) = current_commit {
            // parse numstat line: e.g. "10\t2\tfile.rs"
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let file_path = parts[2];
                
                // Extension Filtering
                let extension = file_path.rsplit('.').next().unwrap_or("");
                let is_valid_extension = config.extensions.is_empty() || config.extensions.iter().any(|ext| ext == extension);
                
                if is_valid_extension && !filter.should_exclude(file_path) {
                    commit_touches_valid_file = true;
                    let ins = parts[0].parse::<u32>().unwrap_or(0);
                    let del = parts[1].parse::<u32>().unwrap_or(0);
                    
                    commit.insertions += ins;
                    commit.deletions += del;
                    commit.changes.push(FileChange {
                        path: file_path.to_string(),
                        insertions: ins,
                        deletions: del,
                    });
                }
            }
        }
    }

    if let Some(commit) = current_commit {
        if commit_touches_valid_file {
            commits.push(commit);
        }
    }

    commits
}

pub fn compute_author_stats(commits: &[Commit]) -> Vec<AuthorStats> {
    let mut stats_map: HashMap<String, AuthorStats> = HashMap::new();

    for commit in commits {
        let key = commit.author_email.clone();
        let stat = stats_map.entry(key).or_insert(AuthorStats {
            name: commit.author_name.clone(),
            email: commit.author_email.clone(),
            commits: 0,
            insertions: 0,
            deletions: 0,
        });

        stat.commits += 1;
        stat.insertions += commit.insertions;
        stat.deletions += commit.deletions;
    }

    let mut result: Vec<AuthorStats> = stats_map.into_values().collect();
    result.sort_by(|a, b| b.commits.cmp(&a.commits));
    result
}

pub fn compute_file_stats(commits: &[Commit]) -> Vec<FileStats> {
    let mut stats_map: HashMap<String, FileStats> = HashMap::new();

    for commit in commits {
        for change in &commit.changes {
            let stat = stats_map.entry(change.path.clone()).or_insert(FileStats {
                path: change.path.clone(),
                commits: 0,
                insertions: 0,
                deletions: 0,
            });

            stat.commits += 1;
            stat.insertions += change.insertions;
            stat.deletions += change.deletions;
        }
    }

    let mut result: Vec<FileStats> = stats_map.into_values().collect();
    // Sort by frequency of changes (commits)
    result.sort_by(|a, b| b.commits.cmp(&a.commits));
    result
}
