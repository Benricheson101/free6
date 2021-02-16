use diesel::{Insertable, Queryable};

use crate::schema::users;

#[derive(Debug, Queryable)]
pub struct User {
    pub id: i32,
    pub user_id: i64,
    pub guild_id: i64,
    pub xp: i32,
}

#[derive(Debug, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub user_id: i64,
    pub guild_id: i64,
}
