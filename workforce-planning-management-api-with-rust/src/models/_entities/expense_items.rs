//! `SeaORM` Entity — `expense_items`. One dated, categorised line on an expense claim (WPM-R55).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "expense_items")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub claim_pid: Uuid,
    pub worker_pid: Uuid,
    pub incurred_on: Date,
    pub category: String,
    pub amount_minor: i64,
    pub description: Option<String>,
    pub receipt_ref: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
