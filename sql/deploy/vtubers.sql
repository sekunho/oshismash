-- Deploy vtubersmash:vtubers to pg

BEGIN;
  CREATE SCHEMA app;

  CREATE TYPE app.TYPE AS ENUM ('vshojo', 'hololive', 'nijisanji', 'indie');

  CREATE TABLE app.vtubers (
    vtuber_id   BIGINT GENERATED ALWAYS AS IDENTITY,
    name        TEXT NOT NULL,
    description TEXT NOT NULL,
    type        app.TYPE NOT NULL,
    img         TEXT
  );
COMMIT;
