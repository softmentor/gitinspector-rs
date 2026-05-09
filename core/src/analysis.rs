use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::config::Config;
use crate::filtering::{Filter, FilterType};
use chrono::{Datelike, TimeZone, Utc};

/// Represents a change to a specific file in a commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub path: String,
    pub insertions: u32,
    pub deletions: u32,
}

/// Represents a single git commit with its metadata and file changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub hash: String,
    pub author_name: String,
    pub author_email: String,
    pub date: String, // Unix timestamp as string
    pub subject: String,
    pub insertions: u32,
    pub deletions: u32,
    pub changes: Vec<FileChange>,
}

/// Aggregated statistics for a specific author.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthorStats {
    pub name: String,
    pub email: String,
    pub commits: u32,
    pub insertions: u32,
    pub deletions: u32,
    /// Activity aggregated by week.
    pub activity: Vec<WeeklyActivity>,
    /// Activity aggregated by day for heatmap visualization.
    pub daily_activity: Vec<DayActivity>,
}

/// Commits per specific day.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayActivity {
    pub date: String, // YYYY-MM-DD
    pub commits: u32,
}

impl AuthorStats {
    /// Computes the net impact (insertions - deletions).
    pub fn impact(&self) -> i64 {
        (self.insertions as i64) - (self.deletions as i64)
    }
}

/// Aggregated activity for a specific week.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WeeklyActivity {
    pub week_id: String, // e.g. "2023-42"
    pub commits: u32,
    pub insertions: u32,
    pub deletions: u32,
    pub top_files: Vec<FileChange>,
}

/// Metadata for a specific branch.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BranchInfo {
    pub name: String,
    pub last_commit_date: String,
    pub last_author: String,
    pub is_stale: bool,
}

/// Aggregated health metrics for the entire repository.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RepoHealth {
    pub active_branches: Vec<BranchInfo>,
    pub stale_branches_count: usize,
    pub estimated_prs_count: usize,
    pub large_files: Vec<FileStats>,
}

/// Statistics for a specific file across all commits.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FileStats {
    pub path: String,
    pub commits: u32,
    pub insertions: u32,
    pub deletions: u32,
    pub total_lines: u32, // Reusing for file size in bytes
    pub loc: u32,         // Actual line count
    pub last_updated: String,
}

impl FileStats {
    /// Extracts the filename (basename) from the full path.
    pub fn filename(&self) -> String {
        std::path::Path::new(&self.path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| self.path.clone())
    }
}

/// Parses raw `git log --numstat` output into a vector of `Commit` objects.
/// 
/// This function handles:
/// - Commit metadata (hash, author, date, subject)
/// - File-level changes (insertions, deletions)
/// - Filtering by extension and exclusion patterns
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

                // Apply advanced filtering
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

/// Aggregates a list of commits into author-specific statistics.
/// 
/// This includes:
/// - Total counts (commits, insertions, deletions)
/// - Weekly activity trends
/// - Daily activity counts for heatmaps
pub fn compute_author_stats(commits: &[Commit]) -> Vec<AuthorStats> {
    let mut stats_map: HashMap<String, AuthorStats> = HashMap::new();
    let mut activity_map: HashMap<String, HashMap<String, WeeklyActivity>> = HashMap::new();
    let mut daily_map: HashMap<String, HashMap<String, u32>> = HashMap::new();

    for commit in commits {
        let key = &commit.author_email;
        let stat = stats_map.entry(key.clone()).or_insert_with(|| AuthorStats {
            name: commit.author_name.clone(),
            email: commit.author_email.clone(),
            commits: 0,
            insertions: 0,
            deletions: 0,
            activity: Vec::new(),
            daily_activity: Vec::new(),
        });

        stat.commits += 1;
        stat.insertions += commit.insertions;
        stat.deletions += commit.deletions;

        // Parse date once for both maps
        if let Ok(ts) = commit.date.parse::<i64>() {
            if let Some(dt) = Utc.timestamp_opt(ts, 0).single() {
                // Daily activity (Heatmap)
                let date_str = dt.format("%Y-%m-%d").to_string();
                let author_daily = daily_map.entry(key.clone()).or_insert_with(HashMap::new);
                *author_daily.entry(date_str).or_insert(0) += 1;

                // Weekly activity (Timeline)
                let week_id = format!("{}-{}", dt.year(), dt.iso_week().week());
                let author_activity = activity_map.entry(key.clone()).or_insert_with(HashMap::new);
                let week_stat = author_activity.entry(week_id.clone()).or_insert_with(|| WeeklyActivity {
                    week_id,
                    commits: 0,
                    insertions: 0,
                    deletions: 0,
                    top_files: Vec::new(),
                });

                week_stat.commits += 1;
                week_stat.insertions += commit.insertions;
                week_stat.deletions += commit.deletions;
                
                // Aggregated weekly file impact
                for change in &commit.changes {
                    if let Some(existing) = week_stat.top_files.iter_mut().find(|f| f.path == change.path) {
                        existing.insertions += change.insertions;
                        existing.deletions += change.deletions;
                    } else {
                        week_stat.top_files.push(change.clone());
                    }
                }
            }
        }
    }

    let mut result: Vec<AuthorStats> = stats_map.into_values().collect();
    for stat in &mut result {
        if let Some(author_activity) = activity_map.remove(&stat.email) {
            let mut activities: Vec<WeeklyActivity> = author_activity.into_values().collect();
            activities.sort_by(|a, b| a.week_id.cmp(&b.week_id));
            
            // Refine weekly top files
            for week in &mut activities {
                week.top_files.sort_by(|a, b| (b.insertions + b.deletions).cmp(&(a.insertions + a.deletions)));
                week.top_files.truncate(15);
            }
            stat.activity = activities;
        }
        
        if let Some(author_daily) = daily_map.remove(&stat.email) {
            let mut days: Vec<DayActivity> = author_daily.into_iter()
                .map(|(date, commits)| DayActivity { date, commits })
                .collect();
            days.sort_by(|a, b| a.date.cmp(&b.date));
            stat.daily_activity = days;
        }
    }
    
    result.sort_by(|a, b| b.commits.cmp(&a.commits));
    result
}

/// Aggregates commit data into file-specific statistics for hotspot analysis.
pub fn compute_file_stats(commits: &[Commit]) -> Vec<FileStats> {
    let mut stats_map: HashMap<String, FileStats> = HashMap::new();

    for commit in commits {
        for change in &commit.changes {
            let stat = stats_map.entry(change.path.clone()).or_insert_with(|| FileStats {
                path: change.path.clone(),
                commits: 0,
                insertions: 0,
                deletions: 0,
                total_lines: 0,
                loc: 0,
                last_updated: String::new(),
            });

            stat.commits += 1;
            stat.insertions += change.insertions;
            stat.deletions += change.deletions;
            
            // Track the most recent commit date for this file
            if commit.date > stat.last_updated {
                stat.last_updated = commit.date.clone();
            }
        }
    }

    let mut result: Vec<FileStats> = stats_map.into_values().collect();
    
    // Convert Unix timestamps to human-readable strings
    for stat in &mut result {
        if !stat.last_updated.is_empty() {
            if let Ok(ts) = stat.last_updated.parse::<i64>() {
                if let Some(dt) = Utc.timestamp_opt(ts, 0).single() {
                    stat.last_updated = dt.format("%b %d, %Y").to_string();
                }
            }
        }
    }

    result.sort_by(|a, b| b.commits.cmp(&a.commits));
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filtering::Filter;

    #[test]
    fn test_parse_commits_basic() {
        let raw = "commit|hash123|Author Name|email@example.com|1600000000|Subject\n10\t5\tfile.rs";
        let config = Config::default();
        let filter = Filter::new(&[]).unwrap();
        let commits = parse_commits(raw, &config, &filter);

        assert_eq!(commits.len(), 1, "Should have parsed exactly one commit");
        assert_eq!(commits[0].author_name, "Author Name");
        assert_eq!(commits[0].insertions, 10);
        assert_eq!(commits[0].deletions, 5);
        assert_eq!(commits[0].changes.len(), 1);
        assert_eq!(commits[0].changes[0].path, "file.rs");
    }

    #[test]
    fn test_compute_author_stats() {
        let commits = vec![Commit {
            hash: "h1".into(),
            author_name: "User".into(),
            author_email: "user@test.com".into(),
            date: "1600000000".into(),
            subject: "Fix".into(),
            insertions: 10,
            deletions: 2,
            changes: vec![FileChange { path: "a.rs".into(), insertions: 10, deletions: 2 }],
        }];

        let stats = compute_author_stats(&commits);
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].commits, 1);
        assert_eq!(stats[0].impact(), 8);
        assert!(!stats[0].daily_activity.is_empty());
    }
}
