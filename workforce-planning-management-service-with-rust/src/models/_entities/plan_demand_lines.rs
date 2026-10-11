//! `SeaORM` Entity — `plan_demand_lines`. Planned headcount for a department (and optionally a role) at a date.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "plan_demand_lines")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub plan_pid: Uuid,
    pub department: String,
    pub role_profile_pid: Option<Uuid>,
    pub target_on: Date,
    pub target_headcount: i32,
    pub note: Option<String>,
    /// The programme the demand serves (`thing:` URN), if any.
    pub programme_ref: Option<String>,
    /// FTE target in percent of one person across the headcount; `None` is unknown, not zero.
    pub target_fte_percent: Option<i32>,
    /// The engagement basis of the demand.
    pub basis: String,
    /// The last day the demanded engagements run, for a basis that ends.
    pub expected_ends_on: Option<Date>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
