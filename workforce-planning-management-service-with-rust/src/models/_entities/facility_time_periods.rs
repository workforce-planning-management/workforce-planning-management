//! `SeaORM` Entity — `facility_time_periods`. Minutes released for representative duties, per recognised body per period (WPM-R107). There is no
//! person here and no kind of time entry for it: a budget can be agreed without a list of representatives (WPM-D68).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "facility_time_periods")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub body_pid: Uuid,
    pub period_start: Date,
    pub period_end: Date,
    pub minutes: i32,
    pub budget_minutes: Option<i32>,
    pub recorded_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
