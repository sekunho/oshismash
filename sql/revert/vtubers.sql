-- Revert oshismash:vtubers from pg

BEGIN;
  DROP FUNCTION app.get_current_and_next;
  DROP FUNCTION app.vote;

  -- DROP INDEX guest_id_index;
  -- DROP INDEX vtuber_id_index;

  DROP TABLE app.guest_votes;
  DROP TABLE app.vtubers;
  DROP TABLE app.groups;
  DROP TABLE app.orgs;
  DROP TYPE app.ACTION;
  DROP TYPE app.REGION;
COMMIT;
