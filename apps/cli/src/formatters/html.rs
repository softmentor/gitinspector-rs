use askama::Template;
use super::Report;
use gitinspector_core::analysis::{AuthorStats, FileStats};
use gitinspector_core::timeline::TimelinePeriod;
use gitinspector_core::blame::BlameStats;

#[derive(Template)]
#[template(path = "report.html")]
struct ReportTemplate<'a> {
    repo_name: &'a String,
    branch_name: &'a String,
    remote_url: &'a Option<String>,
    authors: &'a Vec<AuthorStats>,
    file_stats: &'a Vec<FileStats>,
    _timeline: &'a Option<Vec<TimelinePeriod>>,
    blame: &'a Option<Vec<BlameStats>>,
    health: &'a Option<gitinspector_core::analysis::RepoHealth>,
    version: &'a String,
    duration: &'a String,
    // We pass JSON strings to the template for Chart.js
    timeline_json: String,
    blame_json: String,
    authors_json: String,
}

pub fn format(report: &Report) -> String {
    let timeline_json = serde_json::to_string(&report.timeline).unwrap_or_else(|_| "null".to_string());
    let blame_json = serde_json::to_string(&report.blame).unwrap_or_else(|_| "null".to_string());
    let authors_json = serde_json::to_string(&report.authors).unwrap_or_else(|_| "null".to_string());

    let template = ReportTemplate {
        repo_name: &report.repo_name,
        branch_name: &report.branch_name,
        remote_url: &report.remote_url,
        authors: &report.authors,
        file_stats: &report.file_stats,
        _timeline: &report.timeline,
        blame: &report.blame,
        health: &report.health,
        version: &report.version,
        duration: &report.duration,
        timeline_json,
        blame_json,
        authors_json,
    };

    template.render().unwrap_or_else(|e| format!("Error rendering HTML: {}", e))
}
