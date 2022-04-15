-- Deploy vtubersmash:vtubers to pg

BEGIN;
  CREATE SCHEMA app;

  -- I'm only doing the top vtuber companies + popular indie vtubers for now.
  -- I am but one person.
  CREATE TYPE app.ORIGIN AS ENUM ('vshojo', 'hololive', 'nijisanji', 'indie');

  -- TODO: Fill this out
  CREATE TYPE app.CATEGORY AS ENUM ('holomyth');
  CREATE TYPE app.ACTION AS ENUM ('smashed', 'passed');

  CREATE TABLE app.vtubers (
    vtuber_id   BIGINT GENERATED ALWAYS AS IDENTITY,
    name        TEXT NOT NULL,
    description TEXT NOT NULL,
    origin      app.origin NOT NULL,
    img         TEXT,
    smashes     BIGINT NOT NULL DEFAULT 0 CHECK (smashes >= 0),
    passes      BIGINT NOT NULL DEFAULT 0 CHECK (passes >= 0)
  );

  CREATE FUNCTION app.vote(vtuber_id BIGINT, action app.ACTION)
    RETURNS TABLE (vtuber_id BIGINT, smashes BIGINT, passes BIGINT)
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
      RETURNING vtubers.vtuber_id, vtubers.smashes, vtubers.passes;
    $$;

    COMMENT ON FUNCTION app.vote IS
      'Votes on a VTuber depending on what the action is.';
COMMIT;
