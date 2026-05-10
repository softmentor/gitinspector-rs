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
    pub activity: Vec<WeeklyActivity>,
    pub daily_activity: Vec<DayActivity>,
}

/// Commits per specific day.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayActivity {
    pub date: String, // YYYY-MM-DD
    pub commits: u32,
}

impl AuthorStats {
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
    pub total_lines: u32, 
    pub loc: u32,         
    pub last_updated: String,
}

impl FileStats {
    pub fn filename(&self) -> String {
        std::path::Path::new(&self.path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| self.path.clone())
    }
}

// --- NEW STREAMING COMPONENTS ---

pub struct CommitParser<'a> {
    config: &'a Config,
    filter: &'a Filter,
    current_commit: Option<Commit>,
    commit_touches_valid_file: bool,
}

impl<'a> CommitParser<'a> {
    pub fn new(config: &'a Config, filter: &'a Filter) -> Self {
        Self {
            config,
            filter,
            current_commit: None,
            commit_touches_valid_file: false,
        }
    }

    /// Processes a single line of git log output.
    /// Returns Some(Commit) if a commit was just finalized.
    pub fn parse_line(&mut self, line: &str) -> Option<Commit> {
        let line = line.trim();
        if line.is_empty() {
            return None;
        }

        if line.starts_with("commit|") {
            let previous = self.current_commit.take();
            let touched = self.commit_touches_valid_file;
            
            // Start new commit
            let parts: Vec<&str> = line.splitn(6, '|').collect();
            if parts.len() == 6 {
                let hash = parts[1].to_string();
                let author_name = parts[2].to_string();
                let author_email = parts[3].to_string();
                let date = parts[4].to_string();
                let subject = parts[5].to_string();

                if self.filter.is_excluded(&hash, FilterType::Revision) ||
                   self.filter.is_excluded(&author_name, FilterType::Author) ||
                   self.filter.is_excluded(&author_email, FilterType::Email) ||
                   self.filter.is_excluded(&subject, FilterType::Message) {
                    self.current_commit = None;
                    self.commit_touches_valid_file = false;
                } else {
                    self.current_commit = Some(Commit {
                        hash, author_name, author_email, date, subject,
                        insertions: 0, deletions: 0, changes: Vec::new(),
                    });
                    self.commit_touches_valid_file = false;
                }
            }
            
            if touched { return previous; }
            return None;
        } else if let Some(ref mut commit) = self.current_commit {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let file_path = parts[2];
                let extension = file_path.rsplit('.').next().unwrap_or("");
                let is_valid_extension = self.config.extensions.is_empty() || 
                                       self.config.extensions.iter().any(|ext| ext == extension);
                
                if is_valid_extension && !self.filter.should_exclude(file_path) {
                    self.commit_touches_valid_file = true;
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
        None
    }

    pub fn finalize(&mut self) -> Option<Commit> {
        if self.commit_touches_valid_file {
            self.current_commit.take()
        } else {
            None
        }
    }
}

pub struct IncrementalAggregator {
    stats_map: HashMap<String, AuthorStats>,
    file_stats_map: HashMap<String, FileStats>,
    activity_map: HashMap<String, HashMap<String, WeeklyActivity>>,
    daily_map: HashMap<String, HashMap<String, u32>>,
}

impl IncrementalAggregator {
    pub fn new() -> Self {
        Self {
            stats_map: HashMap::new(),
            file_stats_map: HashMap::new(),
            activity_map: HashMap::new(),
            daily_map: HashMap::new(),
        }
    }

    pub fn add_commit(&mut self, commit: &Commit) {
        let key = &commit.author_email;
        let stat = self.stats_map.entry(key.clone()).or_insert_with(|| AuthorStats {
            name: commit.author_name.clone(),
            email: commit.author_email.clone(),
            ..Default::default()
        });

        stat.commits += 1;
        stat.insertions += commit.insertions;
        stat.deletions += commit.deletions;

        if let Ok(ts) = commit.date.parse::<i64>() {
            if let Some(dt) = Utc.timestamp_opt(ts, 0).single() {
                // Daily
                let date_str = dt.format("%Y-%m-%d").to_string();
                let author_daily = self.daily_map.entry(key.clone()).or_insert_with(HashMap::new);
                *author_daily.entry(date_str).or_insert(0) += 1;

                // Weekly
                let week_id = format!("{}-{}", dt.year(), dt.iso_week().week());
                let author_activity = self.activity_map.entry(key.clone()).or_insert_with(HashMap::new);
                let week_stat = author_activity.entry(week_id.clone()).or_insert_with(|| WeeklyActivity {
                    week_id, ..Default::default()
                });

                week_stat.commits += 1;
                week_stat.insertions += commit.insertions;
                week_stat.deletions += commit.deletions;
                
                for change in &commit.changes {
                    if let Some(existing) = week_stat.top_files.iter_mut().find(|f| f.path == change.path) {
                        existing.insertions += change.insertions;
                        existing.deletions += change.deletions;
                    } else {
                        week_stat.top_files.push(change.clone());
                    }
                    
                    // Prune top files per week to keep memory constant
                    if week_stat.top_files.len() > 30 {
                         week_stat.top_files.sort_by(|a, b| (b.insertions + b.deletions).cmp(&(a.insertions + a.deletions)));
                         week_stat.top_files.truncate(15);
                    }

                    // Global File Stats
                    let fstat = self.file_stats_map.entry(change.path.clone()).or_insert_with(|| FileStats {
                        path: change.path.clone(), ..Default::default()
                    });
                    fstat.commits += 1;
                    fstat.insertions += change.insertions;
                    fstat.deletions += change.deletions;
                    if commit.date > fstat.last_updated {
                        fstat.last_updated = commit.date.clone();
                    }
                }
            }
        }
    }

    pub fn finalize(mut self) -> (Vec<AuthorStats>, Vec<FileStats>) {
        let mut authors: Vec<AuthorStats> = self.stats_map.into_values().collect();
        for stat in &mut authors {
            if let Some(author_activity) = self.activity_map.remove(&stat.email) {
                let mut activities: Vec<WeeklyActivity> = author_activity.into_values().collect();
                activities.sort_by(|a, b| a.week_id.cmp(&b.week_id));
                for week in &mut activities {
                    week.top_files.sort_by(|a, b| (b.insertions + b.deletions).cmp(&(a.insertions + a.deletions)));
                    week.top_files.truncate(15);
                }
                stat.activity = activities;
            }
            if let Some(author_daily) = self.daily_map.remove(&stat.email) {
                let mut days: Vec<DayActivity> = author_daily.into_iter()
                    .map(|(date, commits)| DayActivity { date, commits }).collect();
                days.sort_by(|a, b| a.date.cmp(&b.date));
                stat.daily_activity = days;
            }
        }
        authors.sort_by(|a, b| b.commits.cmp(&a.commits));

        let mut files: Vec<FileStats> = self.file_stats_map.into_values().collect();
        for stat in &mut files {
            if let Ok(ts) = stat.last_updated.parse::<i64>() {
                if let Some(dt) = Utc.timestamp_opt(ts, 0).single() {
                    stat.last_updated = dt.format("%b %d, %Y").to_string();
                }
            }
        }
        files.sort_by(|a, b| b.commits.cmp(&a.commits));

        (authors, files)
    }
}

/// Legacy support for batch processing (internalizes the stream).
pub fn parse_commits(raw_output: &str, config: &Config, filter: &Filter) -> Vec<Commit> {
    let mut parser = CommitParser::new(config, filter);
    let mut commits = Vec::new();
    for line in raw_output.lines() {
        if let Some(commit) = parser.parse_line(line) {
            commits.push(commit);
        }
    }
    if let Some(commit) = parser.finalize() {
        commits.push(commit);
    }
    commits
}

pub fn compute_author_stats(commits: &[Commit]) -> Vec<AuthorStats> {
    let mut agg = IncrementalAggregator::new();
    for c in commits { agg.add_commit(c); }
    agg.finalize().0
}

pub fn compute_file_stats(commits: &[Commit]) -> Vec<FileStats> {
    let mut agg = IncrementalAggregator::new();
    for c in commits { agg.add_commit(c); }
    agg.finalize().1
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
    fn test_incremental_aggregator() {
        let mut agg = IncrementalAggregator::new();
        let commit = Commit {
            hash: "h1".into(),
            author_name: "User".into(),
            author_email: "user@test.com".into(),
            date: "1600000000".into(),
            subject: "Fix".into(),
            insertions: 10,
            deletions: 2,
            changes: vec![FileChange { path: "a.rs".into(), insertions: 10, deletions: 2 }],
        };
        agg.add_commit(&commit);
        let (stats, _) = agg.finalize();
        
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].commits, 1);
        assert_eq!(stats[0].impact(), 8);
        assert!(!stats[0].daily_activity.is_empty());
    }

    #[test]
    fn test_parser_streaming_logic() {
        let config = Config::default();
        let filter = Filter::new(&[]).unwrap();
        let mut parser = CommitParser::new(&config, &filter);
        
        let line1 = "commit|h1|A|e|100|S";
        let line2 = "5\t0\tf1.rs";
        let line3 = "commit|h2|B|e2|200|S2";
        let line4 = "1\t0\tf2.rs";
        
        assert!(parser.parse_line(line1).is_none());
        assert!(parser.parse_line(line2).is_none());
        
        // When the second commit starts, the first one should be returned
        let c1 = parser.parse_line(line3).expect("Should return first commit");
        assert_eq!(c1.hash, "h1");
        assert_eq!(c1.insertions, 5);
        
        assert!(parser.parse_line(line4).is_none());
        let c2 = parser.finalize().expect("Should return second commit");
        assert_eq!(c2.hash, "h2");
        assert_eq!(c2.insertions, 1);
    }
}
