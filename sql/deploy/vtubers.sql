-- Deploy oshismash:vtubers to pg

BEGIN;
  -- Enums
  CREATE TYPE app.REGION AS ENUM ('cn', 'en', 'jp', 'none');
  CREATE TYPE app.ACTION AS ENUM ('smashed', 'passed');

  -- Tables
  CREATE TABLE app.orgs (
    org_id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name   TEXT NOT NULL
  );

  CREATE TABLE app.groups (
    group_id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    org_id   INTEGER REFERENCES app.orgs NOT NULL,
    name     TEXT NOT NULL
  );

  CREATE TABLE app.vtubers (
    vtuber_id   BIGINT PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    name        TEXT NOT NULL,
    description TEXT NOT NULL,
    img         TEXT,

    org_id      INTEGER REFERENCES app.orgs NOT NULL,
    group_id    INTEGER REFERENCES app.groups,
    region      app.REGION NOT NULL
  );

  CREATE TABLE app.guest_votes(
    action_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action    app.ACTION NOT NULL, vtuber_id BIGINT REFERENCES app.vtubers NOT NULL,
    guest_id  UUID REFERENCES app.guests NOT NULL,
    created_at TIMESTAMPTZ DEFAULT now() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT now() NOT NULL
  );

  CREATE INDEX vtuber_id_index ON app.guest_votes (vtuber_id);
  CREATE INDEX guest_id_index ON app.guest_votes (guest_id);
  CREATE UNIQUE INDEX vtuber_guest_index ON app.guest_votes (vtuber_id, guest_id);

  -- Functions
  CREATE OR REPLACE FUNCTION app.vote(guest_id UUID, vtuber_id BIGINT, action app.ACTION)
    RETURNS TABLE (
      vtuber_id BIGINT,
      name TEXT,
      description TEXT,
      img TEXT,
      smashes BIGINT,
      passes BIGINT
    )
    LANGUAGE SQL
    AS $$
      INSERT
        INTO app.guest_votes (guest_id, vtuber_id, action)
        VALUES ($1, $2, $3)
        ON CONFLICT (vtuber_id, guest_id)
            DO UPDATE SET action = $3;

      -- NOTE: I think it's possible to use partitioning here.
      WITH smashes_cte AS (
        SELECT count(*) AS smashes
          FROM app.guest_votes
          WHERE vtuber_id = $2
            AND action = 'smashed' :: app.ACTION
      ), passes_cte AS (
        SELECT count(*) AS passes
          FROM app.guest_votes
          WHERE vtuber_id = $2
            AND action = 'passed' :: app.ACTION
      )
      SELECT
          vtubers.vtuber_id,
          vtubers.name,
          vtubers.description,
          vtubers.img,

          smashes_cte.smashes AS smashes,
          passes_cte.passes AS passes
        FROM
          app.vtubers AS vtubers,
          smashes_cte,
          passes_cte
        WHERE vtubers.vtuber_id = $2;
    $$;

    COMMENT ON FUNCTION app.vote IS
      'Votes on a VTuber depending on what the action is.';

    CREATE FUNCTION app.get_current_and_next(vtuber_id BIGINT)
      RETURNS TABLE (
        vtuber_id BIGINT,
        name TEXT,
        description TEXT,
        img TEXT
      )
      LANGUAGE SQL
      AS $$
        SELECT vtuber_id, name, description, img
          FROM app.vtubers;
      $$;
COMMIT;
