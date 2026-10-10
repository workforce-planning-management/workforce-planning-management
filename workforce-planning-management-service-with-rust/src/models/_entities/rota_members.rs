//! `SeaORM` Entity — `rota_members`. One member of a rota, in rotation order.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "rota_members")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub rota_pid: Uuid,
    pub worker_pid: Uuid,
    pub position: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
