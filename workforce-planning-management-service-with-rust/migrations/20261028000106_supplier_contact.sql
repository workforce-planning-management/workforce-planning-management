-- Migration: **the supplier's contact** on a contractor's details (WPM-R206). A `person:` URN (the contact-relationship-management
-- app keys its contacts by `person_ref`). WPM stores the reference and nothing about the contact: the name is looked up from the owner.

ALTER TABLE worker_contractor_details ADD COLUMN IF NOT EXISTS supplier_contact_ref VARCHAR NULL
                     CHECK (supplier_contact_ref IS NULL OR supplier_contact_ref ~ '^person:[0-9a-f-]{36}$');
