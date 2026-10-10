//! `SeaORM` Entity — `expense_claims`. A worker's request to be repaid for money they spent (WPM-R55).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "expense_claims")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub worker_pid: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub currency: String,
    pub status: String,
    pub submitted_at: Option<DateTimeWithTimeZone>,
    pub decided_by: Option<String>,
    pub decided_at: Option<DateTimeWithTimeZone>,
    pub decision_note: Option<String>,
    pub reimbursed_on: Option<Date>,
    pub reimbursed_by: Option<String>,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
    pub deleted_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
