use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use crate::analysis::Commit;

/// Represents a period in the timeline (e.g., "2023-W01" or "2023-01")
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinePeriod {
    pub period_name: String,
    pub commits: u32,
    pub insertions: u32,
    pub deletions: u32,
}

/// Generates a timeline from a list of commits.
/// Grouped by weeks if `use_weeks` is true, otherwise by months.
pub fn generate_timeline(commits: &[Commit], use_weeks: bool) -> Vec<TimelinePeriod> {
    let mut periods: BTreeMap<String, TimelinePeriod> = BTreeMap::new();

    for commit in commits {
        if let Ok(ts) = commit.date.parse::<i64>() {
            use chrono::{TimeZone, Utc};
            let dt = Utc.timestamp_opt(ts, 0).unwrap();
            let date = dt.date_naive();
            
            let period_key = if use_weeks {
                format!("{:04}-W{:02}", date.year(), date.iso_week().week())
            } else {
                format!("{:04}-{:02}", date.year(), date.month())
            };

            let entry = periods.entry(period_key.clone()).or_insert(TimelinePeriod {
                period_name: period_key,
                commits: 0,
                insertions: 0,
                deletions: 0,
            });

            entry.commits += 1;
            entry.insertions += commit.insertions;
            entry.deletions += commit.deletions;
        }
    }

    periods.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_timeline_months() {
        let commits = vec![
            Commit { hash: "1".into(), author_name: "".into(), author_email: "".into(), date: "1673740800".into(), subject: "".into(), insertions: 10, deletions: 5, changes: vec![] }, // Jan 15 2023
            Commit { hash: "2".into(), author_name: "".into(), author_email: "".into(), date: "1674172800".into(), subject: "".into(), insertions: 2, deletions: 1, changes: vec![] }, // Jan 20 2023
            Commit { hash: "3".into(), author_name: "".into(), author_email: "".into(), date: "1675555200".into(), subject: "".into(), insertions: 5, deletions: 0, changes: vec![] }, // Feb 05 2023
        ];

        let timeline = generate_timeline(&commits, false); // use_weeks = false
        assert_eq!(timeline.len(), 2);
        assert_eq!(timeline[0].period_name, "2023-01");
        assert_eq!(timeline[0].commits, 2);
        assert_eq!(timeline[0].insertions, 12);
        
        assert_eq!(timeline[1].period_name, "2023-02");
        assert_eq!(timeline[1].commits, 1);
    }
}
