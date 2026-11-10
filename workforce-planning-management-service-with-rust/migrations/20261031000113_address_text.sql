-- Migration: a worker's address as **one free-text block** (WPM-D152, WPM-T382), beside the structured fields that
-- stay. The UI offers the block for a country its address order and label table lacks; the person writes the
-- address in their own order. Nothing existing is rewritten: a row's structured fields are kept as they are.

ALTER TABLE worker_contact_details ADD COLUMN IF NOT EXISTS address_text TEXT NULL;
