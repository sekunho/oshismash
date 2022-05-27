-- Deploy oshismash:vtubers to pg

BEGIN;
  -- Enums
  CREATE TYPE app.REGION AS ENUM ('cn', 'en', 'jp', 'none');
  CREATE TYPE app.ACTION AS ENUM ('smashed', 'passed');

  -- Tables CREATE TABLE app.orgs (
    org_id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name   TEXT NOT NULL
  );

  CREATE TABLE app.groups (
    group_id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    org_id   INTEGER REFERENCES app.orgs NOT NULL,
    name     TEXT NOT NULL
  );

  CREATE INDEX groups_org_index ON app.groups (org_id);

  CREATE TABLE app.vtubers (
    vtuber_id   BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name        TEXT NOT NULL,
    description TEXT NOT NULL,
    img         TEXT,

    org_id      INTEGER REFERENCES app.orgs NOT NULL,
    group_id    INTEGER REFERENCES app.groups,
    region      app.REGION NOT NULL,

    prev        BIGINT REFERENCES app.vtubers (vtuber_id),
    next        BIGINT REFERENCES app.vtubers (vtuber_id)
  );

  CREATE INDEX vtubers_org_index ON app.vtubers (org_id);
  CREATE INDEX vtubers_group_index ON app.vtubers (group_id);

  CREATE TABLE app.guest_votes(
    action_id  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action     app.ACTION NOT NULL, vtuber_id BIGINT REFERENCES app.vtubers NOT NULL,
    guest_id   UUID REFERENCES app.guests NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT now() NOT NULL
  );

  CREATE INDEX vtuber_id_index ON app.guest_votes (vtuber_id);
  CREATE INDEX guest_id_index ON app.guest_votes (guest_id);
  CREATE UNIQUE INDEX vtuber_guest_index ON app.guest_votes (vtuber_id, guest_id);

  CREATE FUNCTION app.get_metrics(vtuber_id BIGINT)
    RETURNS TABLE (smashes BIGINT, passes BIGINT)
    LANGUAGE SQL
    AS $$
      WITH smashes_cte AS (
        SELECT count(*)
          FROM app.guest_votes
          WHERE guest_votes.vtuber_id = $1
            AND guest_votes.action = 'smashed'
      ), passes_cte AS (
        SELECT count(*)
          FROM app.guest_votes
          WHERE guest_votes.vtuber_id = $1
            AND guest_votes.action = 'passed'
      )
      SELECT
        smashes_cte.count AS smashes,
        passes_cte.count AS passes
      FROM smashes_cte, passes_cte, app.vtubers
      GROUP BY smashes_cte.count, passes_cte.count;
    $$;

  CREATE OR REPLACE FUNCTION app.get_vote_stack_from_previous(
    prev_vtuber_id BIGINT
  )
    RETURNS JSONB
    LANGUAGE SQL
    AS $$
      WITH current_cte AS (
        SELECT vtubers.next
          FROM app.vtubers
          WHERE vtubers.vtuber_id = $1
      )
      SELECT app.get_vote_stack($1, current_cte.next)
        FROM current_cte;
    $$;

  CREATE OR REPLACE FUNCTION app.get_vote_stack_from_current(
    current_vtuber_id BIGINT
  )
    RETURNS JSONB
    LANGUAGE SQL
    AS $$
      WITH current_cte AS (
        SELECT vtubers.prev, vtubers.vtuber_id
          FROM app.vtubers
          WHERE vtubers.vtuber_id = $1
      )
      SELECT app.get_vote_stack(current_cte.prev, current_cte.vtuber_id)
        FROM current_cte;
    $$;


  CREATE OR REPLACE FUNCTION app.get_vote_stack
    ( prev_vtuber_id BIGINT
    , current_vtuber_id BIGINT
    )
    RETURNS JSONB
    LANGUAGE PLPGSQL
    AS $$
      DECLARE
        data JSONB;
      BEGIN
        IF prev_vtuber_id IS NULL AND current_vtuber_id IS NULL THEN
          RAISE SQLSTATE 'Z0001'
            USING MESSAGE = 'Arguments should not be both NULL';
        END IF;

        WITH current_vtuber_cte AS (
          -- I'm using jsonb_agg to bypass the annoyance of when `current_vtuber`
          -- is `NULL`. If it is, using `current_vtuber` in the `FROM` clause
          -- at the next query is going to cause the entire result to be `NULL`.
          -- This is not what I want since I have to return the VTuber that was
          -- previously voted.
          SELECT jsonb_agg(
              json_build_object
                ( 'id'
                , vtubers.vtuber_id
                , 'description'
                , vtubers.description
                , 'name'
                , vtubers.name
                , 'prev'
                , vtubers.prev
                , 'next'
                , vtubers.next
                , 'img'
                , vtubers.img
                , 'org_name'
                , orgs.name
                )
            )
            FROM app.vtubers AS vtubers JOIN app.orgs AS orgs
            ON vtubers.org_id = orgs.org_id
            WHERE vtubers.vtuber_id = get_vote_stack.current_vtuber_id
        ), prev_results_cte AS (
          -- Grabs the results of the previous VTuber relative to the current.
          -- This includes the smash/pass metrics.
          SELECT
            jsonb_agg(
              json_build_object
                ( 'vtuber_id'
                , vtubers.vtuber_id
                , 'name'
                , vtubers.name
                , 'description'
                , vtubers.description
                , 'img'
                , vtubers.img
                , 'next'
                , vtubers.next
                , 'prev'
                , vtubers.prev
                , 'smashes'
                , metrics.smashes
                , 'passes'
                , metrics.passes
                )
            )
            FROM app.vtubers AS vtubers
               , app.get_metrics(prev_vtuber_id) AS metrics
            WHERE vtubers.vtuber_id = get_vote_stack.prev_vtuber_id
        )
        SELECT
          json_build_object
            ( 'current'
            , current_vtuber_cte.jsonb_agg -> 0
            , 'results'
            , prev_results_cte.jsonb_agg -> 0
            )
          INTO data
          FROM current_vtuber_cte, prev_results_cte;
        RETURN data;
      END;
    $$;

    COMMENT ON FUNCTION app.get_vote_stack IS
      'Gets the current details of the VTuber, and the information + vote results of the previous VTuber.';

  -- Functions
  CREATE OR REPLACE FUNCTION app.vote(guest_id UUID, vtuber_id BIGINT, action app.ACTION)
    RETURNS JSONB
    LANGUAGE SQL
    AS $$
      -- Perform an upsert. Users are allowed to change their votes in the event
      -- of, er, a phenomenon that clears their mind after doing a specific
      -- action. Maybe they will be filled with regret, or something, who knows.
      --
      -- If there's a conflict, then this updates the vote to the new one.
      WITH prev_vtuber_cte AS (
        INSERT
          INTO app.guest_votes (guest_id, vtuber_id, action)
          VALUES ($1, $2, $3)
          ON CONFLICT (vtuber_id, guest_id)
              DO UPDATE SET action = $3
          RETURNING vtuber_id AS prev_vtuber_id
      ), current_vtuber_cte AS (
        SELECT next AS current_vtuber_id
          FROM app.vtubers, prev_vtuber_cte
          WHERE vtubers.vtuber_id = prev_vtuber_cte.prev_vtuber_id
      )
      SELECT app.get_vote_stack(prev_vtuber_cte.prev_vtuber_id, current_vtuber_cte.current_vtuber_id)
        FROM prev_vtuber_cte, current_vtuber_cte;
    $$;

    COMMENT ON FUNCTION app.vote IS
      'Votes on a VTuber depending on what the action is.';
COMMIT;
