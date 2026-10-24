//! `SeaORM` Entity — `positions`. A budgeted seat (WPM-R152, WPM-D104): cost centre, budget in FTE percent, funding kind and
//! state, with the holder, the requisition and the plan demand line as links. No pay.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "positions")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub organization_ref: String,
    pub cost_centre: String,
    pub title: String,
    pub budget_fte_percent: i32,
    pub funding_kind: String,
    pub funding_ends_on: Option<Date>,
    pub programme_ref: Option<String>,
    pub state: String,
    pub holder_worker_pid: Option<Uuid>,
    pub requisition_pid: Option<Uuid>,
    pub demand_line_pid: Option<Uuid>,
    pub recorded_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
