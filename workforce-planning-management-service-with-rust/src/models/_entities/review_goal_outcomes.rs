//! `SeaORM` Entity — `review_goal_outcomes`. A goal in a review: the outcome the reviewer and the reviewee each record, and the snapshot frozen when the review is shared (WPM-R225, WPM-D165).

#![allow(missing_docs)]

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "review_goal_outcomes")]
pub struct Model {
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub pid: Uuid,
    pub review_pid: Uuid,
    pub goal_pid: Uuid,
    pub worker_pid: Uuid,
    pub reviewer_outcome: Option<String>,
    pub reviewee_outcome: Option<String>,
    pub snapshot: Option<Json>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
