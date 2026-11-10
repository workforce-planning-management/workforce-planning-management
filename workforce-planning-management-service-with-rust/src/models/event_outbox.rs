//! `event_outbox` model — the transactional-outbox write surface for
//! the durable event bus (Phase 2; `agents/share/event-bus.md` §3).
//!
//! [`OutboxInsert::from_envelope`] is the **pure** envelope→row mapping
//! (unit-testable DB-free). [`crate::streaming::emit_in`] performs the
//! in-transaction `INSERT` on the handler's own connection, giving the
//! entity write and the event one commit boundary. The Phase-3 relay's
//! poll and ack are roadmap (the relay itself is not built yet).

use axa::prelude::*;
use chrono::{DateTime, FixedOffset};
use uuid::Uuid;

use crate::streaming::Envelope;

/// The column values for one outbox row, derived from a canonical
/// [`Envelope`] plus the enqueue timestamp. A plain struct so the
/// derivation is unit-testable DB-free.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutboxInsert {
    /// Envelope dedup id.
    pub event_id: Uuid,
    /// Entity name (`worker` / `requisition` / …).
    pub entity: String,
    /// Record pid (bus partition key).
    pub entity_pid: Uuid,
    /// Change kind token (`created`, `worker_hired`, …).
    pub kind: String,
    /// When the change occurred.
    pub occurred_at: DateTime<FixedOffset>,
    /// Actor pid, if any.
    pub actor: Option<String>,
    /// Envelope schema version.
    pub schema_version: i32,
    /// The full envelope as JSON.
    pub payload: serde_json::Value,
}

impl OutboxInsert {
    /// Derive the outbox row from an envelope, stamping `occurred_at`.
    /// Pure and DB-free: no clock, no connection.
    ///
    /// # Errors
    ///
    /// When `env.pid` is not a UUID, or the envelope fails to serialize.
    pub fn from_envelope(env: &Envelope, occurred_at: DateTime<FixedOffset>) -> ModelResult<Self> {
        let entity_pid = Uuid::parse_str(&env.pid).map_err(|e| ModelError::Any(Box::new(e)))?;
        let payload = serde_json::to_value(env).map_err(|e| ModelError::Any(Box::new(e)))?;
        Ok(Self {
            event_id: env.event_id,
            entity: env.entity.clone(),
            entity_pid,
            kind: env.kind.clone(),
            occurred_at,
            actor: env.actor.clone(),
            schema_version: i32::try_from(env.schema_version).unwrap_or(1),
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::OutboxInsert;
    use crate::streaming::Envelope;
    use chrono::{FixedOffset, TimeZone};
    use uuid::Uuid;

    fn an_envelope(kind: &str, pid: &str) -> Envelope {
        Envelope {
            event_id: Uuid::parse_str("22222222-2222-4222-8222-222222222222").unwrap(),
            schema_version: crate::streaming::SCHEMA_VERSION,
            entity: "worker".to_string(),
            kind: kind.to_string(),
            pid: pid.to_string(),
            seq: 7,
            actor: Some("user-1".to_string()),
            name: "E-1001".to_string(),
            data: None,
        }
    }

    fn an_instant() -> chrono::DateTime<FixedOffset> {
        FixedOffset::east_opt(0)
            .unwrap()
            .with_ymd_and_hms(2026, 7, 17, 9, 0, 0)
            .unwrap()
    }

    /// Every field maps and the full envelope survives as the payload.
    #[test]
    fn from_envelope_maps_every_column_and_keeps_the_full_payload() {
        let pid = "0c4f1e2a-0000-4000-8000-000000000000";
        let env = an_envelope("worker_hired", pid);
        let row = OutboxInsert::from_envelope(&env, an_instant()).unwrap();
        assert_eq!(row.event_id, env.event_id);
        assert_eq!(row.entity, "worker");
        assert_eq!(row.entity_pid, Uuid::parse_str(pid).unwrap());
        assert_eq!(row.kind, "worker_hired");
        assert_eq!(row.actor.as_deref(), Some("user-1"));
        assert_eq!(row.payload["pid"], pid);
        assert_eq!(row.payload["kind"], "worker_hired");
    }

    /// A non-UUID pid is rejected, never panics.
    #[test]
    fn from_envelope_rejects_a_non_uuid_pid() {
        assert!(
            OutboxInsert::from_envelope(&an_envelope("created", "not-a-uuid"), an_instant())
                .is_err()
        );
    }
}
