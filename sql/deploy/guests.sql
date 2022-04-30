-- Deploy oshismash:guests to pg

BEGIN;

  CREATE TABLE app.guests(
    guest_id   UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ DEFAULT now(),
    updated_at TIMESTAMPTZ DEFAULT now()
  );

  CREATE FUNCTION app.verify_guest(guest_id UUID)
    RETURNS BOOLEAN
    LANGUAGE SQL
    AS $$
      SELECT * FROM
    $$;

  CREATE FUNCTION app.create_guest()
    RETURNS TABLE (guest_id TEXT)
    LANGUAGE SQL
    AS $$
      INSERT
        INTO app.guests
        DEFAULT VALUES
        RETURNING guest_id;
    $$;

  CREATE FUNCTION app.update_guest_time(guest_id UUID)
    RETURNS TABLE (updated_at TIMESTAMPTZ)
    LANGUAGE SQL
    AS $$
      UPDATE app.guests
        SET updated_at = now()
        WHERE guest_id = $1
        RETURNING updated_at;
    $$;

COMMIT;
