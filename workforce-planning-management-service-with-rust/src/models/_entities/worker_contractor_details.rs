//! `SeaORM` Entity — `worker_contractor_details`. A contractor's supplier, route and rate (WPM-R80). The rate is masked like salary.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "worker_contractor_details")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub worker_pid: Uuid,
    pub supplier_ref: Option<String>,
    pub route: Option<String>,
    pub rate_minor: Option<i64>,
    pub rate_currency: Option<String>,
    pub rate_basis: Option<String>,
    pub recorded_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
