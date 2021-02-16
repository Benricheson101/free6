use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
    result::Error as DieselError,
    PgConnection,
    QueryDsl,
    RunQueryDsl,
};
use serenity::model::id::{GuildId, UserId};

use crate::{
    models::user::{NewUser, User},
    schema::users,
};

pub struct Database {
    conn: Pool<ConnectionManager<PgConnection>>,
}

impl Database {
    pub fn new(database_url: &str) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(database_url);

        Self {
            conn: Pool::builder().build(manager).expect("Error creating pool"),
        }
    }

    pub fn create_guild_user(
        &self,
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<User, DieselError> {
        let new_user = NewUser {
            user_id: user_id.0 as i64,
            guild_id: guild_id.0 as i64,
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .get_result(&self.conn.get().unwrap())
    }

    pub fn get_guild_user(
        &self,
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<User, DieselError> {
        users::table
            .filter(users::guild_id.eq(guild_id.0 as i64))
            .filter(users::user_id.eq(user_id.0 as i64))
            .get_result(&self.conn.get().unwrap())
    }

    pub fn get_guild_users(
        &self,
        guild_id: GuildId,
    ) -> Result<Vec<User>, DieselError> {
        users::table
            .filter(users::guild_id.eq(guild_id.0 as i64))
            .get_results(&self.conn.get().unwrap())
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
        .get_result(&self.conn.get().unwrap())
    }

    pub fn add_guild_user_xp(
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
        .set(users::xp.eq(users::xp + xp))
        .get_result(&self.conn.get().unwrap())
    }
}
