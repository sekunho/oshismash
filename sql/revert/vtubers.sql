-- Revert oshismash:vtubers from pg

BEGIN;
  DROP SCHEMA app CASCADE;
COMMIT;
