//! `SeaORM` Entity — `scim_pushes`. What was last sent to the identity provider for a worker (WPM-R150): the provider's id,
//! a fingerprint and the retry state. No name, no contact detail, no body.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "scim_pushes")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    #[sea_orm(unique)]
    pub worker_pid: Uuid,
    pub scim_id: Option<String>,
    pub body_hash: Option<String>,
    pub pushed_at: Option<DateTimeWithTimeZone>,
    pub attempts: i32,
    pub next_attempt_at: Option<DateTimeWithTimeZone>,
    pub last_error: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
