//! Commit and file analytics split by command domain.

pub mod churn;
pub mod contributors;
pub mod timeline;

pub use churn::{ChurnSummary, FileChurn, file_churn_stats};
pub use contributors::{ContributorStats, contributor_stats};
pub use timeline::{Event, generate_events_from_commits, sort_events};
