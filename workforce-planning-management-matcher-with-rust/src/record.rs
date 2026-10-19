//! The record being matched.
//!
//! A [`WorkerRecord`] is whatever the caller has *right now* about a worker:
//! every field is optional, because an unknown is never a zero. The matcher
//! reads records and keeps nothing; it is the caller's job to hold or discard
//! them under the project's retention rules.
//!
//! **No national identifiers and nothing about health.** WPM stores neither
//! (`AGENTS/privacy-and-data-rules.md`), so the matcher has no field for them.
//! An employer's own worker number is the only identifier, and it is scoped to
//! the organization that issued it.

use std::fmt;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// An employer-issued worker number, scoped to the organization that issued it.
///
/// `("org:a", "1042")` and `("org:b", "1042")` are different workers.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkerNumber {
    /// The issuing organization, as the caller names it (an `EntityRef` URN is typical).
    pub organization: String,
    /// The number as typed; case, spaces and dashes are ignored when comparing.
    pub number: String,
}

impl WorkerNumber {
    pub fn new(organization: impl Into<String>, number: impl Into<String>) -> Self {
        Self {
            organization: organization.into(),
            number: number.into(),
        }
    }
}

/// A worker as one source system describes them.
#[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WorkerRecord {
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub emails: Vec<String>,
    pub phones: Vec<String>,
    pub worker_numbers: Vec<WorkerNumber>,
}

impl WorkerRecord {
    /// An empty record; chain the `with_*` methods to fill it.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_given_name(mut self, v: impl Into<String>) -> Self {
        self.given_name = Some(v.into());
        self
    }

    pub fn with_family_name(mut self, v: impl Into<String>) -> Self {
        self.family_name = Some(v.into());
        self
    }

    /// `None` when the calendar date does not exist (for example 31 February).
    pub fn with_date_of_birth(mut self, year: i32, month: u32, day: u32) -> Option<Self> {
        self.date_of_birth = Some(NaiveDate::from_ymd_opt(year, month, day)?);
        Some(self)
    }

    pub fn with_email(mut self, v: impl Into<String>) -> Self {
        self.emails.push(v.into());
        self
    }

    pub fn with_phone(mut self, v: impl Into<String>) -> Self {
        self.phones.push(v.into());
        self
    }

    pub fn with_worker_number(
        mut self,
        organization: impl Into<String>,
        number: impl Into<String>,
    ) -> Self {
        self.worker_numbers
            .push(WorkerNumber::new(organization, number));
        self
    }
}

/// Debug shows which fields are present, never their values: a stray `{:?}` in
/// a log line must not leak a name, a birth date or a contact detail.
impl fmt::Debug for WorkerRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkerRecord")
            .field("given_name", &self.given_name.as_ref().map(|_| "…"))
            .field("family_name", &self.family_name.as_ref().map(|_| "…"))
            .field("date_of_birth", &self.date_of_birth.map(|_| "…"))
            .field("emails", &self.emails.len())
            .field("phones", &self.phones.len())
            .field("worker_numbers", &self.worker_numbers.len())
            .finish()
    }
}

impl fmt::Debug for WorkerNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("WorkerNumber(…)")
    }
}
