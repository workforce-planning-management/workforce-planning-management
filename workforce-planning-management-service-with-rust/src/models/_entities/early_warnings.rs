//! `SeaORM` Entity — `early_warnings`. A condition that became true (a pool over-committed, a measure in breach): when it was first seen, when it stopped, when a person acknowledged it (WPM-R68). Names a pool or a measure, never a person.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "early_warnings")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub organization_ref: String,
    pub kind: String,
    pub subject_pid: Uuid,
    pub subject_name: String,
    pub first_true_on: Date,
    pub cleared_on: Option<Date>,
    pub acknowledged_on: Option<Date>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
