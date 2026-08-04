-- Shared BEFORE UPDATE trigger function for tables with an `updated_at` column.
CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Attach (or re-attach) the trigger to every public table that has `updated_at`.
-- After adding `updated_at` to a new table, re-run this block (or add a migration
-- that calls the same DO $$ ... $$) so the trigger is wired automatically.
DO $$
DECLARE
    tbl RECORD;
    trigger_name TEXT;
BEGIN
    FOR tbl IN
        SELECT c.relname AS table_name
        FROM pg_class c
        JOIN pg_namespace n ON n.oid = c.relnamespace
        JOIN pg_attribute a ON a.attrelid = c.oid
        WHERE n.nspname = 'public'
          AND c.relkind = 'r'
          AND a.attname = 'updated_at'
          AND a.attnum > 0
          AND NOT a.attisdropped
    LOOP
        trigger_name := tbl.table_name || '_set_updated_at';
        EXECUTE format('DROP TRIGGER IF EXISTS %I ON %I', trigger_name, tbl.table_name);
        EXECUTE format(
            'CREATE TRIGGER %I BEFORE UPDATE ON %I FOR EACH ROW EXECUTE FUNCTION set_updated_at()',
            trigger_name,
            tbl.table_name
        );
    END LOOP;
END;
$$;
