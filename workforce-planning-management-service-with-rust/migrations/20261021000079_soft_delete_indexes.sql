-- Migration: an index on `deleted_at` for every soft-deleting table (WPM-R142). The retention report counts, and the
-- sweep deletes, the rows past their horizon; with no index each is a scan of the whole table, and `time_entries`
-- alone holds millions of rows at the supported size. Only the deleted rows are indexed, so the index is small.

DO $$
                 DECLARE t text;
                 BEGIN
                     FOR t IN
                         SELECT c.table_name FROM information_schema.columns c
                           JOIN information_schema.tables x
                             ON x.table_schema = c.table_schema AND x.table_name = c.table_name
                          WHERE c.column_name = 'deleted_at' AND c.table_schema = current_schema()
                            AND x.table_type = 'BASE TABLE'
                     LOOP
                         EXECUTE format(
                             'CREATE INDEX IF NOT EXISTS %I ON %I (deleted_at) WHERE deleted_at IS NOT NULL',
                             t || '_deleted_at_idx', t);
                     END LOOP;
                 END $$;
