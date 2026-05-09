use serde::{Deserialize, Serialize};

pub const DEFAULT_EXTENSIONS: &[&str] = &["java", "c", "cc", "cpp", "h", "hh", "hpp", "py", "glsl", "rb", "js", "sql", "rs", "ts", "tsx", "jsx", "go", "swift", "kt", "md"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub repo_path: String,
    pub hard: bool,
    pub use_weeks: bool,
    pub include_metrics: bool,
    pub list_file_types: bool,
    pub responsibilities: bool,
    pub timeline: bool,
    pub since: Option<String>,
    pub until: Option<String>,
    pub extensions: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            repo_path: ".".to_string(),
            hard: false,
            use_weeks: false,
            include_metrics: false,
            list_file_types: false,
            responsibilities: false,
            timeline: false,
            since: None,
            until: None,
            extensions: DEFAULT_EXTENSIONS.iter().map(|s| s.to_string()).collect(),
        }
    }
}
