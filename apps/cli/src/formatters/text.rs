use super::Report;

pub fn format(report: &Report) -> String {
    let mut out = String::new();
    
    out.push_str(&format!("Repository: {}\n", report.repo_name));
    out.push_str(&format!("Branch:     {}\n", report.branch_name));
    if let Some(url) = &report.remote_url {
        out.push_str(&format!("Remote:     {}\n", url));
    }
    out.push_str(&format!("Found {} authors.\n\n", report.authors.len()));
    
    out.push_str(&format!("{:<25} | {:>10} | {:>15} | {:>15}\n", "Author", "Commits", "Insertions", "Deletions"));
    out.push_str(&format!("{:-<25}-|-{:-<10}-|-{:-<15}-|-{:-<15}\n", "", "", "", ""));
    for stat in &report.authors {
        out.push_str(&format!("{:<25} | {:>10} | {:>15} | {:>15}\n", stat.name, stat.commits, stat.insertions, stat.deletions));
    }
    out.push('\n');

    if let Some(timeline) = &report.timeline {
        out.push_str("--- Timeline ---\n");
        out.push_str(&format!("{:<10} | {:>10} | {:>10} | {:>10}\n", "Period", "Commits", "Insertions", "Deletions"));
        out.push_str(&format!("{:-<10}-|-{:-<10}-|-{:-<10}-|-{:-<10}\n", "", "", "", ""));
        for period in timeline {
            out.push_str(&format!("{:<10} | {:>10} | {:>10} | {:>10}\n", period.period_name, period.commits, period.insertions, period.deletions));
        }
        out.push('\n');
    }

    if let Some(blame) = &report.blame {
        out.push_str("--- Blame Responsibilities ---\n");
        for stat in blame {
            out.push_str(&format!("\n{} ({} lines)\n", stat.author_name, stat.lines));
            for file in &stat.top_files {
                out.push_str(&format!("  {:<40} | {:>10}\n", file.path, file.lines));
            }
        }
        out.push('\n');
    }

    if report.metrics_enabled {
        out.push_str("--- Metrics ---\n");
        out.push_str("Cyclomatic complexity feature is active.\n");
    }

    out
}
