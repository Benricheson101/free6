-- Your SQL goes here
CREATE TABLE users (
  id SERIAL PRIMARY KEY,
  user_id BIGINT NOT NULL,
  guild_id BIGINT NOT NULL,
  xp INTEGER NOT NULL DEFAULT 0,
  blocked BOOLEAN DEFAULT 'f' NOT NULL,
  UNIQUE (user_id, guild_id)
)
