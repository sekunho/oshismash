BEGIN;

SELECT plan(3);

--------------------------------------------------------------------------------

DELETE FROM app.vtubers;
DELETE FROM app.orgs;
DELETE FROM app.groups;

-- Relaxes the restriction of setting IDs.
ALTER TABLE app.vtubers
  ALTER vtuber_id SET GENERATED BY DEFAULT;

ALTER TABLE app.orgs
  ALTER org_id SET GENERATED BY DEFAULT;

ALTER TABLE app.groups
ALTER group_id SET GENERATED BY DEFAULT;

INSERT
  INTO app.orgs(org_id, name)
  VALUES (1 :: INTEGER, 'vshojo');

INSERT
  INTO app.vtubers(vtuber_id, name, description, org_id, region)
  VALUES (1 :: BIGINT, 'Veibae', 'A succubus', 1 :: INTEGER, 'none' :: app.REGION);

SELECT row_eq(
  $$ SELECT vtuber_id, smashes, passes FROM app.vtubers $$,
  row(1 :: BIGINT, 0 :: BIGINT, 0 :: BIGINT)
);

-- smash action
SELECT row_eq(
  $$ SELECT vtuber_id, smashes, passes FROM app.vote(1 :: BIGINT, 'smashed' :: app.ACTION) $$,
  row(1 :: BIGINT, 1 :: BIGINT, 0 :: BIGINT)
);

-- pass action
SELECT row_eq(
  $$
    SELECT vtuber_id, smashes, passes
      FROM app.vote(1 :: BIGINT, 'passed' :: app.ACTION)
  $$,
  row(1 :: BIGINT, 1 :: BIGINT, 1 :: BIGINT)
);


--------------------------------------------------------------------------------

SELECT * FROM finish();

ROLLBACK;
