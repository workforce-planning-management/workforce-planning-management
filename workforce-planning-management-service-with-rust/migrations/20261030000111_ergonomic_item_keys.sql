-- Migration: **a default checklist item is stored by key** (WPM-T386).
--
-- `ergonomic_items.item_key` holds the stable key of a default workstation item (`chair`, `lighting`, …), which a locale translates;
-- `name` keeps the English text for an API reader, and a custom item has no key. Rows written before this migration hold only the
-- English sentence: those whose `name` is exactly one of the eight default sentences get its key, so they show in the reader's language
-- too; any other row is left as typed. The pairs are the service's `rules::ergonomics::DSE_ITEMS`, and a unit test there fails if they
-- differ.

ALTER TABLE ergonomic_items ADD COLUMN IF NOT EXISTS item_key VARCHAR NULL
                 CONSTRAINT ergonomic_items_item_key_shape CHECK (item_key ~ '^[a-z][a-z_]*$');

UPDATE ergonomic_items SET item_key = 'screen' WHERE item_key IS NULL AND name = 'Screen at a comfortable height and distance, free of flicker';

UPDATE ergonomic_items SET item_key = 'chair' WHERE item_key IS NULL AND name = 'Chair adjustable, with lower-back support';

UPDATE ergonomic_items SET item_key = 'keyboard_mouse' WHERE item_key IS NULL AND name = 'Keyboard and mouse positioned to keep wrists straight';

UPDATE ergonomic_items SET item_key = 'desk_space' WHERE item_key IS NULL AND name = 'Enough desk space; documents holdable beside the screen';

UPDATE ergonomic_items SET item_key = 'lighting' WHERE item_key IS NULL AND name = 'Lighting adequate; screen free of glare and reflections';

UPDATE ergonomic_items SET item_key = 'leg_room' WHERE item_key IS NULL AND name = 'Leg room clear; feet supported';

UPDATE ergonomic_items SET item_key = 'breaks' WHERE item_key IS NULL AND name = 'Work pattern allows breaks or changes of activity';

UPDATE ergonomic_items SET item_key = 'software' WHERE item_key IS NULL AND name = 'Software and display legible and responsive';
