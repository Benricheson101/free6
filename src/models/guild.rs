use diesel::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

use crate::schema::guilds;

#[derive(Debug, Queryable, Deserialize, Serialize)]
pub struct Guild {
    pub id: i32,
    pub guild_id: i64,
    pub prefix: String,
}

#[derive(Debug, Insertable)]
#[table_name = "guilds"]
pub struct NewGuild {
    pub guild_id: i64,
}
