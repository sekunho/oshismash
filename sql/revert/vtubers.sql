-- Revert oshismash:vtubers from pg

BEGIN;
  DROP FUNCTION app.get_vtuber_results;
  DROP FUNCTION app.get_metrics;
  DROP FUNCTION app.get_vote_stack_from_current;
  DROP FUNCTION app.get_vote_stack_from_previous;
  DROP FUNCTION app.get_vote_stack;
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
