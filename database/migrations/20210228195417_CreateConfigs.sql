-- Add migration script here
CREATE TABLE configs (
  id SERIAL PRIMARY KEY,
  guild_id BIGINT UNIQUE NOT NULL,
  --  prefix VARCHAR(32) DEFAULT '~' NOT NULL,
  locale VARCHAR(3) DEFAULT 'en' NOT NULL
);
