//! `SeaORM` Entity — `coverage_reminders`. A coverage-expiry reminder already sent (WPM-R220, WPM-T420), keyed by record, end date and kind.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "coverage_reminders")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub coverage_pid: Uuid,
    pub valid_to: Date,
    pub kind: String,
    pub reminded_on: Date,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
