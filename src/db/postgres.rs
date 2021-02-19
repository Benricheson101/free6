use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    result::Error as DieselError,
    PgConnection,
    QueryDsl,
    RunQueryDsl,
};
use serenity::model::id::{GuildId, UserId};

use super::redis::RedisCache;
use crate::{
    models::{
        guild::{Guild, NewGuild},
        user::{NewUser, User},
    },
    schema::{guilds, users},
};

/// The main DB for the bot
pub struct Database {
    pool: Pool<ConnectionManager<PgConnection>>,
    redis: RedisCache,
}

impl Database {
    pub fn new(database_url: &str, redis: RedisCache) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(database_url);

        Self {
            pool: Pool::builder()
                .build(manager)
                .expect("Error creating postgres pool"),
            redis,
        }
    }

    // -- users --

    /// Calls Database::create_guild_user_with_xp with 0 XP
    pub fn create_guild_user(
        &self,
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<User, DieselError> {
        self.create_guild_user_with_xp(user_id, guild_id, 0)
    }

    /// Create a guild user with XP
    ///
    /// # SQL:
    /// ```sql
    /// INSERT INTO users (user_id, guild_id, blocked, xp)
    /// VALUES (...);
    /// ```
    pub fn create_guild_user_with_xp(
        &self,
        user_id: UserId,
        guild_id: GuildId,
        xp: i32,
    ) -> Result<User, DieselError> {
        let new_user = NewUser {
            user_id: user_id.0 as i64,
            guild_id: guild_id.0 as i64,
            blocked: false,
            xp,
        };

        let u = diesel::insert_into(users::table)
            .values(&new_user)
            .get_result(&self.pool.get().unwrap())?;

        self.redis.set_user(&u);

        Ok(u)
    }

    /// Get a guild user from redis and postgres
    ///
    /// # SQL:
    /// ```sql
    /// SELECT * FROM users
    /// WHERE guild_id = <guild_id> AND user_id = <user_id>;
    /// ```
    pub fn get_guild_user(
        &self,
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<User, DieselError> {
        if let Some(user) = self.redis.get_user(&guild_id, &user_id) {
            Ok(user)
        } else {
            users::table
                .filter(users::guild_id.eq(guild_id.0 as i64))
                .filter(users::user_id.eq(user_id.0 as i64))
                .get_result(&self.pool.get().unwrap())
        }
    }

    /// Get all guild users from redis and postgres
    ///
    /// # SQL:
    /// ```sql
    /// SELECT * FROM users
    /// WHERE (users.user_id, users.guild_id) NOT IN (
    ///     (uid, gid)
    /// );
    /// ```
    pub fn get_guild_users(
        &self,
        guild_id: GuildId,
    ) -> Result<Vec<User>, DieselError> {
        users::table
            .filter(users::guild_id.eq(guild_id.0 as i64))
            .get_results(&self.pool.get().unwrap())
    }

    /// Set an *existing* user's XP
    ///
    /// # SQL:
    /// ```sql
    /// UPDATE users
    /// SET xp = <xp>
    /// WHERE guild_id = <guild_id> and user_id = <user_id>;
    /// ```
    pub fn set_guild_user_xp(
        &self,
        user_id: UserId,
        guild_id: GuildId,
        xp: i32,
    ) -> Result<User, DieselError> {
        self.redis.del_user(&guild_id, &user_id);

        let user = diesel::update(
            users::table
                .filter(users::guild_id.eq(guild_id.0 as i64))
                .filter(users::user_id.eq(user_id.0 as i64)),
        )
        .set(users::xp.eq(xp))
        .get_result(&self.pool.get().unwrap())?;

        self.redis.set_user(&user);

        Ok(user)
    }

    /// Update a user's XP or create a row in the users table
    ///
    /// # SQL:
    /// ```sql
    /// INSERT INTO users (user_id, guild_id, xp, blocked)
    /// VALUES (...)
    /// ON CONFLICT (user_id, guild_id)
    /// DO
    ///     UPDATE SET xp = <xp>;
    /// ```
    pub fn add_guild_user_xp(
        &self,
        user_id: UserId,
        guild_id: GuildId,
        xp: i32,
    ) -> Result<User, DieselError> {
        self.redis.del_user(&guild_id, &user_id);

        let new_user = NewUser {
            user_id: user_id.0 as i64,
            guild_id: guild_id.0 as i64,
            blocked: false,
            xp,
        };

        let user = diesel::insert_into(users::table)
            .values(&new_user)
            .on_conflict((users::user_id, users::guild_id))
            .do_update()
            .set(users::xp.eq(users::xp + xp))
            .get_result(&self.pool.get().unwrap())?;

        self.redis.set_user(&user);

        Ok(user)
    }

    pub fn top_n_guild_user_xp(
        &self,
        guild_id: GuildId,
        n: i64,
    ) -> Result<Vec<User>, DieselError> {
        users::table
            .filter(users::guild_id.eq(guild_id.0 as i64))
            .order(users::xp.desc())
            .limit(n)
            .get_results(&self.pool.get().unwrap())
    }

    // -- guilds --

    pub fn create_guild(
        &self,
        guild_id: GuildId,
    ) -> Result<Guild, DieselError> {
        let new_guild = NewGuild {
            guild_id: guild_id.0 as i64,
            prefix: "~".to_string(),
        };

        diesel::insert_into(guilds::table)
            .values(&new_guild)
            .get_result(&self.pool.get().unwrap())
    }

    pub fn get_guild(&self, guild_id: GuildId) -> Result<Guild, DieselError> {
        if let Some(from_redis) = self.redis.get_guild(guild_id) {
            Ok(from_redis)
        } else {
            let guild = guilds::table
                .filter(guilds::guild_id.eq(guild_id.0 as i64))
                .get_result(&self.pool.get().unwrap())?;

            self.redis.set_guild(&guild);

            Ok(guild)
        }
    }

    pub fn get_guild_prefix(
        &self,
        guild_id: GuildId,
    ) -> Result<String, DieselError> {
        if let Some(from_redis) = self.redis.get_guild(guild_id) {
            Ok(from_redis.prefix)
        } else {
            let guild = guilds::table
                .filter(guilds::guild_id.eq(guild_id.0 as i64))
                .get_result(&self.pool.get().unwrap())?;

            self.redis.set_guild(&guild);

            Ok(guild.prefix)
        }
    }

    pub fn set_guild_prefix(
        &self,
        guild_id: GuildId,
        prefix: String,
    ) -> Result<Guild, DieselError> {
        let new_guild = NewGuild {
            guild_id: guild_id.0 as i64,
            prefix: prefix.clone(),
        };

        self.redis.del_guild(&guild_id);

        let saved = diesel::insert_into(guilds::table)
            .values(&new_guild)
            .on_conflict(guilds::guild_id)
            .do_update()
            .set(guilds::prefix.eq(prefix))
            .get_result(&self.pool.get().unwrap())?;

        self.redis.set_guild(&saved);

        Ok(saved)
    }
}
