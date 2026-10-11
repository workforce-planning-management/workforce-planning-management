//! `SeaORM` Entity — `idempotency_keys`. A caller's key, the fingerprint of the request that claimed it and the response to replay; kept for one calendar day (WPM-R132).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "idempotency_keys")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    pub actor: String,
    pub idem_key: String,
    pub fingerprint: String,
    pub status_code: Option<i32>,
    pub content_type: Option<String>,
    #[serde(skip)]
    pub response_body: Option<Vec<u8>>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
