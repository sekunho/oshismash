-- Deploy oshismash:vtubers to pg

BEGIN;
  CREATE SCHEMA app;

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
    region      app.REGION NOT NULL,

    smashes     BIGINT NOT NULL DEFAULT 0 CHECK (smashes >= 0),
    passes      BIGINT NOT NULL DEFAULT 0 CHECK (passes >= 0)
  );

  -- Functions
  CREATE FUNCTION app.vote(vtuber_id BIGINT, action app.ACTION)
    RETURNS TABLE (
      vtuber_id BIGINT,
      name TEXT,
      img TEXT,
      smashes BIGINT,
      passes BIGINT
    )
    LANGUAGE SQL
    AS $$
      UPDATE app.vtubers
      SET
        smashes = (
          CASE action
            WHEN ('smashed' :: app.ACTION)
              THEN smashes + 1
            ELSE smashes
          END
        ),
        passes = (
          CASE action
            WHEN ('passed' :: app.ACTION)
              THEN passes + 1
            ELSE passes
          END
        )
      WHERE vtubers.vtuber_id = vtuber_id
      RETURNING
        vtubers.vtuber_id,
        vtubers.name,
        vtubers.img,
        vtubers.smashes,
        vtubers.passes;
    $$;

    COMMENT ON FUNCTION app.vote IS
      'Votes on a VTuber depending on what the action is.';
COMMIT;
