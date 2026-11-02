//! `SeaORM` Entity — `qualification_reminders`. That the person and their manager were told about one expiry date of one qualification, once (WPM-T435).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "qualification_reminders")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub worker_pid: Uuid,
    pub qualification_pid: Uuid,
    pub expires_on: Date,
    pub kind: String,
    pub reminded_on: Date,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
