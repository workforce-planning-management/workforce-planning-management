//! # Workforce planning management matcher
//!
//! Decides whether two worker records describe the same person, for import
//! de-duplication, rehire detection and merging a directory feed. It is
//! **pure** (no database, no clock, no network), **stateless**, **explainable**
//! (every field's score is in the result) and `Send + Sync`.
//!
//! It follows the project's data rules: an unknown is never a zero, an
//! estimate says what it rests on, and there is no field for a national
//! identifier or anything about health. A [`WorkerRecord`] prints its field
//! counts, not its values, in `Debug`.
//!
//! | Module | Purpose |
//! |---|---|
//! | [`record`] | [`WorkerRecord`], [`WorkerNumber`]: what is matched |
//! | [`normalizer`] | folding, email, E.164-style phone, worker number, Soundex |
//! | [`scorer`] | Jaro-Winkler, Levenshtein, combined |
//! | [`nicknames`] | [`NicknameTable`]: Mike ↔ Michael, extendable |
//! | [`matcher`] | [`MatchingEngine`], [`MatchConfig`], [`MatchResult`] |
//!
//! ```
//! use workforce_planning_management_matcher::{Decision, MatchingEngine, WorkerRecord};
//!
//! let hr = WorkerRecord::new()
//!     .with_given_name("Michael")
//!     .with_family_name("O'Brien")
//!     .with_email("m.obrien@example.org");
//! let feed = WorkerRecord::new()
//!     .with_given_name("Mike")
//!     .with_family_name("OBrien")
//!     .with_email("M.OBrien@Example.org");
//!
//! let result = MatchingEngine::default().match_workers(&hr, &feed);
//! assert_eq!(result.decision, Decision::Match);
//! assert_eq!(result.breakdown.email, Some(1.0));
//! assert_eq!(result.breakdown.date_of_birth, None); // unknown, not zero
//! ```

#![forbid(unsafe_code)]
#![warn(clippy::pedantic)]
#![deny(missing_docs)]

pub mod error;
pub mod matcher;
pub mod nicknames;
pub mod normalizer;
pub mod record;
pub mod scorer;

pub use error::MatchError;
pub use matcher::{
    Basis, Breakdown, Decision, MatchConfig, MatchResult, MatchingEngine, Ranked, Weights,
};
pub use nicknames::NicknameTable;
pub use record::{WorkerNumber, WorkerRecord};
