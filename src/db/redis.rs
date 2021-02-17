use r2d2_redis::{r2d2::Pool, redis::Commands, RedisConnectionManager};
use serenity::model::id::GuildId;
use tracing::debug;

use crate::models::guild::Guild;

const GUILD_TTL_SECS: usize = 10;

pub struct RedisCache {
    pool: Pool<RedisConnectionManager>,
}

// TODO: proper error handling
impl RedisCache {
    pub fn new(redis_url: &str) -> Self {
        let manager = RedisConnectionManager::new(redis_url)
            .expect("Error creating RedisConnectionManager");

        let pool = Pool::builder()
            .build(manager)
            .expect("Error creating redis pool");

        Self { pool }
    }

    pub fn del_guild(&self, guild: &GuildId) {
        debug!("RedisCache#del_guild");
        let _: () = self
            .pool
            .get()
            .unwrap()
            .del(format!("guilds:{}", guild.0))
            .unwrap();
    }

    pub fn get_guild(&self, guild: GuildId) -> Option<Guild> {
        debug!("RedisCache#get_guild");
        let g: Option<String> = self
            .pool
            .get()
            .unwrap()
            .get(format!("guilds:{}", guild.0))
            .unwrap();

        if let Some(g) = g {
            serde_json::from_str(&g).unwrap()
        } else {
            None
        }
    }

    pub fn set_guild(&self, guild: &Guild) {
        debug!("RedisCache#set_guild");
        let json = serde_json::to_string(&guild).unwrap();

        let _: () = self
            .pool
            .get()
            .unwrap()
            .set_ex(format!("guilds:{}", guild.guild_id), json, GUILD_TTL_SECS)
            .unwrap();
    }
}
