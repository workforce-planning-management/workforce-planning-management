//! Row type — `worker_contact_details`. A worker's own address, telephone numbers and personal e-mail (WPM-R124).

#![allow(missing_docs)]

use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub id: i32,
    pub worker_pid: Uuid,
    pub address_line1: Option<String>,
    pub address_line2: Option<String>,
    pub city: Option<String>,
    pub region: Option<String>,
    pub postcode: Option<String>,
    pub address_text: Option<String>,
    pub country: Option<String>,
    pub phone_mobile: Option<String>,
    pub phone_home: Option<String>,
    pub personal_email: Option<String>,
    pub phone_mobile_digits: Option<String>,
    pub phone_mobile_e164: Option<String>,
    pub phone_home_digits: Option<String>,
    pub phone_home_e164: Option<String>,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
}
