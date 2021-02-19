use r2d2_redis::{
    r2d2::Pool,
    redis::{self, Commands},
    RedisConnectionManager,
};
use serenity::model::id::{GuildId, UserId};
use tracing::debug;

use crate::models::{guild::Guild, user::User};

const GUILD_TTL_SECS: usize = 10;
const USER_TTL_SECS: usize = 10;

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
            .del(self.format_guild_key(guild.0))
            .unwrap();
    }

    pub fn get_guild(&self, guild: GuildId) -> Option<Guild> {
        debug!("RedisCache#get_guild");
        let g: Option<String> = self
            .pool
            .get()
            .unwrap()
            .get(self.format_guild_key(guild.0))
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
            .set_ex(
                self.format_guild_key(guild.guild_id as u64),
                json,
                GUILD_TTL_SECS,
            )
            .unwrap();
    }

    // -- users --

    pub fn del_user(&self, guild: &GuildId, user: &UserId) {
        let _: () = self
            .pool
            .get()
            .unwrap()
            .del(self.format_user_key(guild.0, user.0))
            .unwrap();
    }

    pub fn get_user(&self, guild: &GuildId, user: &UserId) -> Option<User> {
        let u: Option<String> = self
            .pool
            .get()
            .unwrap()
            .get(self.format_user_key(guild.0, user.0))
            .unwrap();

        if let Some(u) = u {
            serde_json::from_str(&u).unwrap()
        } else {
            None
        }
    }

    pub fn set_user(&self, user: &User) {
        let json = serde_json::to_string(&user).unwrap();

        let _: () = self
            .pool
            .get()
            .unwrap()
            .set_ex(
                self.format_user_key(user.guild_id as u64, user.user_id as u64),
                json,
                USER_TTL_SECS,
            )
            .unwrap();
    }

    pub fn _get_guild_users(&self, guild: &GuildId) -> Vec<User> {
        let mut pool = self.pool.get().unwrap();
        let keys: redis::Iter<String> =
            pool.scan_match(format!("users:{}:*", guild.0)).unwrap();

        let redis_keys = keys.collect::<Vec<String>>();

        // why
        let from_redis = match redis_keys.len() {
            0 => return vec![],
            1 => {
                let k: String = pool.get(&redis_keys[0]).unwrap();
                vec![k]
            },
            _ => pool.get(redis_keys).unwrap(),
        };

        let users = from_redis
            .iter()
            .map(|u| serde_json::from_str(&u).unwrap())
            .collect::<Vec<User>>();

        println!("{:#?}", users);

        users
    }

    fn format_user_key(&self, guild: u64, user: u64) -> String {
        format!("users:{}:{}", guild, user)
    }

    fn format_guild_key(&self, guild: u64) -> String {
        format!("guilds:{}", guild)
    }
}
