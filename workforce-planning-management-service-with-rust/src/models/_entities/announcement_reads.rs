//! `SeaORM` Entity — `announcement_reads`. That a worker has read an announcement (the reader sees their own; editors see only a count).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "announcement_reads")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub announcement_pid: Uuid,
    pub worker_pid: Uuid,
    pub read_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
