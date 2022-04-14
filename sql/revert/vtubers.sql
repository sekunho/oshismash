-- Revert vtubersmash:vtubers from pg

BEGIN;
  DROP SCHEMA app CASCADE;
COMMIT;
