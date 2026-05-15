//! Commit and file analytics split by command domain.

pub mod churn;
pub mod contributors;
pub mod timeline;

pub use churn::{file_churn_stats, ChurnSummary, FileChurn};
pub use contributors::{contributor_stats, ContributorStats};
pub use timeline::{generate_events_from_commits, sort_events, Event};
