//! `SeaORM` Entity — `conversion_plans`. A recorded intent for a fixed-term or contractor engagement: convert, extend, end or undecided (WPM-R98, WPM-D66). No pay figure.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "conversion_plans")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub intent: String,
    pub target_on: Date,
    pub department: Option<String>,
    pub role_profile_ref: Option<String>,
    pub post_funding_kind: Option<String>,
    pub post_funding_ends_on: Option<Date>,
    pub reason: Option<String>,
    pub proposed_by: Option<String>,
    pub approved_by: Option<String>,
    pub status: String,
    pub review_on: Option<Date>,
    pub settled_on: Option<Date>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
