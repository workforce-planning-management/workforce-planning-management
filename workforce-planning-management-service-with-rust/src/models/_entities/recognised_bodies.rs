//! `SeaORM` Entity — `recognised_bodies`. A union, works council or staff forum the organization recognises (WPM-R102): a name, a kind, a
//! basis, a date and a contact role. No member, representative or contact person is held (WPM-D68).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "recognised_bodies")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub organization_ref: String,
    pub name: String,
    pub kind: String,
    pub basis: String,
    pub recognised_on: Date,
    pub contact_role: String,
    pub recorded_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
