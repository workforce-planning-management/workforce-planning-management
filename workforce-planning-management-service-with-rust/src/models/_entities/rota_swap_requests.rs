//! `SeaORM` Entity — `rota_swap_requests`. A request that a colleague take the requester's on-call days in a window.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "rota_swap_requests")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub rota_pid: Uuid,
    pub requester_pid: Uuid,
    pub taker_pid: Uuid,
    pub starts_on: Date,
    pub ends_on: Date,
    pub note: Option<String>,
    pub status: String,
    pub decided_at: Option<DateTimeWithTimeZone>,
    pub created_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
