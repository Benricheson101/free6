-- Your SQL goes here
CREATE TABLE guilds (
  id SERIAL PRIMARY KEY,
  guild_id BIGINT UNIQUE NOT NULL,
  prefix VARCHAR(32) DEFAULT '~' NOT NULL
)
