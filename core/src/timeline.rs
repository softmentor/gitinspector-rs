use chrono::{NaiveDate, Datelike};
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
        // Assume date is "YYYY-MM-DD"
        if let Ok(date) = NaiveDate::parse_from_str(&commit.date, "%Y-%m-%d") {
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
            Commit { hash: "1".into(), author_name: "".into(), author_email: "".into(), date: "2023-01-15".into(), subject: "".into(), insertions: 10, deletions: 5 },
            Commit { hash: "2".into(), author_name: "".into(), author_email: "".into(), date: "2023-01-20".into(), subject: "".into(), insertions: 2, deletions: 1 },
            Commit { hash: "3".into(), author_name: "".into(), author_email: "".into(), date: "2023-02-05".into(), subject: "".into(), insertions: 5, deletions: 0 },
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
