-- Revert oshismash:guests from pg

BEGIN;

  DROP TABLE app.guests;

  DROP FUNCTION app.create_guest;
  DROP FUNCTION app.update_guest_time;
  DROP FUNCTION app.verify_guest;

  DROP SCHEMA app;

COMMIT;
