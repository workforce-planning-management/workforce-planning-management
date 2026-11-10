-- Migration: **generic pay scale id**. The one reference pay scale is now
-- `national-2026-27`. Until this change the service knew exactly one scale, and
-- it validated every stored id against it, so every stored id is the old id of
-- that same scale: each is moved to the new id. A worker's pay position and a
-- role profile's pay band keep their band and step.

UPDATE worker_pay_positions SET scale_id = 'national-2026-27' WHERE scale_id <> 'national-2026-27';

UPDATE role_profiles SET pay_scale_id = 'national-2026-27' WHERE pay_scale_id IS NOT NULL AND pay_scale_id <> 'national-2026-27';
