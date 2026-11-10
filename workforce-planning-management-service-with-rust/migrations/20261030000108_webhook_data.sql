-- Migration: **what a hire or an ending tells a webhook subscriber** (proposed WPM-D145, WPM-T372). A delivery of `worker_hired`,
-- `worker_terminated` or `worker_retired` keeps the handoff the event carried (`person_ref`, `organization_ref`, `effective_on`, and
-- `from` and `to` for an ending; `rules::bridge::handoff_data`), so the payload can name the person by reference. Any other kind has
-- none, and the schema refuses one. It is erased with the delivery, which names the worker.

ALTER TABLE webhook_deliveries ADD COLUMN IF NOT EXISTS data JSONB NULL
                     CHECK (data IS NULL OR kind IN ('worker_hired', 'worker_terminated', 'worker_retired'));
