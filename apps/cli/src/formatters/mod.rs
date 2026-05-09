use gitinspector_core::analysis::{AuthorStats, FileStats};
use gitinspector_core::timeline::TimelinePeriod;
use gitinspector_core::blame::BlameStats;
use serde::Serialize;

pub mod text;
pub mod json;
pub mod xml;
pub mod html;
pub mod markdown;

#[derive(Serialize)]
pub struct Report {
    pub repo_name: String,
    pub branch_name: String,
    pub remote_url: Option<String>,
    pub authors: Vec<AuthorStats>,
    pub file_stats: Vec<FileStats>,
    pub timeline: Option<Vec<TimelinePeriod>>,
    pub blame: Option<Vec<BlameStats>>,
    pub health: Option<gitinspector_core::analysis::RepoHealth>,
    pub metrics_enabled: bool,
    pub version: String,
    pub duration: String,
}
