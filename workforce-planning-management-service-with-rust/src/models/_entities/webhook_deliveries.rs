//! `SeaORM` Entity — `webhook_deliveries`. One event for one subscription: its kind, the URN it is about and how the attempts
//! went (WPM-R151). No payload body and no personal field.

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "webhook_deliveries")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub subscription_pid: Uuid,
    pub event_id: Uuid,
    pub kind: String,
    pub subject: String,
    pub occurred_at: DateTimeWithTimeZone,
    pub status: String,
    pub attempts: i32,
    pub last_status: Option<i32>,
    pub last_error: Option<String>,
    pub next_attempt_at: Option<DateTimeWithTimeZone>,
    pub delivered_at: Option<DateTimeWithTimeZone>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
