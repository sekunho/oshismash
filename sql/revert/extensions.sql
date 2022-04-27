-- Revert oshismash:extensions.sql from pg

BEGIN;

  DROP EXTENSION pgtap;

COMMIT;
