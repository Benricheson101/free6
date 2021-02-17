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

    pub fn create_guild_user(
        &self,
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<User, DieselError> {
        self.create_guild_user_with_xp(user_id, guild_id, 0)
    }

    pub fn create_guild_user_with_xp(
        &self,
        user_id: UserId,
        guild_id: GuildId,
        xp: i32,
    ) -> Result<User, DieselError> {
        let new_user = NewUser {
            user_id: user_id.0 as i64,
            guild_id: guild_id.0 as i64,
            xp,
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .get_result(&self.pool.get().unwrap())
    }

    pub fn get_guild_user(
        &self,
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<User, DieselError> {
        users::table
            .filter(users::guild_id.eq(guild_id.0 as i64))
            .filter(users::user_id.eq(user_id.0 as i64))
            .get_result(&self.pool.get().unwrap())
    }

    pub fn get_guild_users(
        &self,
        guild_id: GuildId,
    ) -> Result<Vec<User>, DieselError> {
        users::table
            .filter(users::guild_id.eq(guild_id.0 as i64))
            .get_results(&self.pool.get().unwrap())
    }

    pub fn set_guild_user_xp(
        &self,
        user_id: UserId,
        guild_id: GuildId,
        xp: i32,
    ) -> Result<User, DieselError> {
        diesel::update(
            users::table
                .filter(users::guild_id.eq(guild_id.0 as i64))
                .filter(users::user_id.eq(user_id.0 as i64)),
        )
        .set(users::xp.eq(xp))
        .get_result(&self.pool.get().unwrap())
    }

    pub fn add_guild_user_xp(
        &self,
        user_id: UserId,
        guild_id: GuildId,
        xp: i32,
    ) -> Result<User, DieselError> {
        let new_user = NewUser {
            user_id: user_id.0 as i64,
            guild_id: guild_id.0 as i64,
            xp,
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .on_conflict((users::user_id, users::guild_id))
            .do_update()
            .set(users::xp.eq(users::xp + xp))
            .get_result(&self.pool.get().unwrap())
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
}
