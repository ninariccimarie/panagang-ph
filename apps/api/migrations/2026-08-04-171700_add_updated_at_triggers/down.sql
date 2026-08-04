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
    END LOOP;
END;
$$;

DROP FUNCTION IF EXISTS set_updated_at();
