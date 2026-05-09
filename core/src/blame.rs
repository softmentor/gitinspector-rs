use crate::config::Config;
use crate::provider::GitProvider;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::task;
use crate::filtering::{Filter, FilterType};

use serde::{Deserialize, Serialize};

/// Detailed blame statistics per file for an author.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FileBlame {
    pub path: String,
    pub lines: u32,
}

impl FileBlame {
    pub fn filename(&self) -> String {
        std::path::Path::new(&self.path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| self.path.clone())
    }
}

/// Aggregated blame statistics per author.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BlameStats {
    pub author_name: String,
    pub author_email: String,
    pub lines: u32,
    pub top_files: Vec<FileBlame>,
}

/// Computes the blame lines per author by executing `git blame` concurrently.
pub async fn compute_blame(
    provider: Arc<dyn GitProvider + Send + Sync>,
    config: Arc<Config>,
    filter: Arc<Filter>,
    files: Vec<String>,
) -> Result<Vec<BlameStats>, String> {
    let mut tasks = Vec::new();

    for file in files {
        let provider = provider.clone();
        let config = config.clone();
        let filter = filter.clone();
        
        let extension = file.rsplit('.').next().unwrap_or("");
        let is_valid_extension = config.extensions.is_empty() || config.extensions.iter().any(|ext| ext == extension);
        
        if is_valid_extension && !filter.should_exclude(&file) {
            let file_path = file.clone();
            tasks.push(task::spawn(async move {
                match provider.get_blame_file(&config, &file_path).await {
                    Ok(output) => Some((file_path, parse_blame_output(&output, &filter))),
                    Err(_) => None,
                }
            }));
        }
    }

    // author_email -> author_name
    let mut author_names: HashMap<String, String> = HashMap::new();
    // author_email -> { file_path -> line_count }
    let mut author_file_map: HashMap<String, HashMap<String, u32>> = HashMap::new();

    for handle in tasks {
        if let Ok(Some((path, file_stats))) = handle.await {
            for (email, stat) in file_stats {
                author_names.insert(email.clone(), stat.author_name);
                let files_map = author_file_map.entry(email).or_insert_with(HashMap::new);
                *files_map.entry(path.clone()).or_insert(0) += stat.lines;
            }
        }
    }

    let mut result = Vec::new();
    for (email, files_map) in author_file_map {
        let mut top_files: Vec<FileBlame> = files_map.into_iter()
            .map(|(path, lines)| FileBlame { path, lines })
            .collect();
        top_files.sort_by(|a, b| b.lines.cmp(&a.lines));
        top_files.truncate(10); // Keep top 10 files

        let total_lines = top_files.iter().map(|f| f.lines).sum();
        
        result.push(BlameStats {
            author_name: author_names.get(&email).cloned().unwrap_or_default(),
            author_email: email,
            lines: total_lines,
            top_files,
        });
    }

    result.sort_by(|a, b| b.lines.cmp(&a.lines));
    Ok(result)
}

fn parse_blame_output(output: &str, filter: &Filter) -> HashMap<String, BlameStats> {
    let mut stats = HashMap::new();
    let mut current_name = String::new();
    let mut current_email = String::new();
    
    for line in output.lines() {
        if line.starts_with("author ") {
            current_name = line["author ".len()..].to_string();
        } else if line.starts_with("author-mail ") {
            current_email = line["author-mail ".len()..].trim_matches(|c| c == '<' || c == '>').to_string();
        } else if line.starts_with('\t') {
            if filter.is_excluded(&current_name, FilterType::Author) || 
               filter.is_excluded(&current_email, FilterType::Email) {
                continue;
            }

            if !current_email.is_empty() {
                let entry = stats.entry(current_email.clone()).or_insert(BlameStats {
                    author_name: current_name.clone(),
                    author_email: current_email.clone(),
                    lines: 0,
                    top_files: Vec::new(),
                });
                entry.lines += 1;
            }
        }
    }
    stats
}
