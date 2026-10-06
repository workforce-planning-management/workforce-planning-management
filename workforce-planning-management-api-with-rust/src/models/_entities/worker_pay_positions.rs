//! `SeaORM` Entity — `worker_pay_positions`. A worker's band and step on a reference pay scale (WPM-R54).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "worker_pay_positions")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub worker_pid: Uuid,
    pub scale_id: String,
    pub band: String,
    pub step: i32,
    pub step_since: Date,
    pub recorded_by: Option<String>,
    pub on_behalf: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
