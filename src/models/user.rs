use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

use crate::schema::users;

// TODO: rename GuildUser?
#[derive(Debug, Queryable, Deserialize, Serialize)]
pub struct User {
    pub id: i32,
    pub user_id: i64,
    pub guild_id: i64,
    pub xp: i32,
    pub blocked: bool,
}

#[derive(Debug, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub user_id: i64,
    pub guild_id: i64,
    pub xp: i32,
    pub blocked: bool,
}
