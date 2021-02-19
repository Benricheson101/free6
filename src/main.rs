mod cmds;
mod db;
mod hooks;
pub mod models;
pub mod schema;
pub mod util;

use std::{collections::HashSet, env, sync::Arc, time::Duration};

#[macro_use]
extern crate diesel;

use cmds::{meta::*, xp::*};
use db::{postgres::Database, redis::RedisCache};
use dotenv::dotenv;
use fluent_templates::static_loader;
use lru_time_cache::LruCache;
use serenity::{
    async_trait,
    client::{
        bridge::gateway::{GatewayIntents, ShardManager},
        EventHandler,
    },
    framework::{standard::macros::group, StandardFramework},
    http::Http,
    model::prelude::*,
    prelude::*,
};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

pub const MIN_MESSAGE_XP: i32 = 15;
pub const MAX_MESSAGE_XP: i32 = 25;
pub const XP_TIMEOUT_SECS: u64 = 60;

static_loader! {
    pub static LOCALES = {
        locales: "./lang",
        fallback_language: "en",

        customise: |bundle| bundle.set_use_isolating(false),
    };
}


#[macro_export]
macro_rules! args {
    ( $($key:expr => $val:expr),* $(,)? ) => {
        {
            let mut map = std::collections::HashMap::new();
            $(
                map.insert($key, $val.into());
            )*
            map
        }
    }
}

#[group("Meta")]
#[commands(
    ping_cmd,
    github_cmd,
    get_user_cmd,
    create_user_cmd,
    get_all_users_cmd,
    leaderboard_cmd,
    get_user_cache_cmd,
    create_guild_cmd,
    prefix_cmd,
    fluent_test_cmd,
)]
#[description = "Meta commands, idk, nothing too special here"]
struct MetaCmds;

#[group("XP")]
#[commands(set_xp_cmd, rank_cmd)]
#[description = "Commands related to the XP leveling system"]
struct XpCmds;

pub struct ShardManagerContainer;
pub struct MessageXPTimeoutCache;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

impl TypeMapKey for Database {
    type Value = Arc<Mutex<Database>>;
}

impl TypeMapKey for MessageXPTimeoutCache {
    type Value = Arc<Mutex<LruCache<(UserId, GuildId), i32>>>;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        info!(
            "[{}] Ready as {}#{:04} in {} servers!",
            ctx.shard_id,
            ready.user.name,
            ready.user.discriminator,
            ready.guilds.len()
        );
    }

    async fn resume(&self, _ctx: Context, _: ResumedEvent) {
        info!("Resumed.");
    }
}

#[tokio::main]
async fn main() {
    dotenv().expect("Failed to load .env file");

    let subscriber = tracing_subscriber::fmt()
        .pretty()
        .with_target(false)
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to start the logger");

    let token = env::var("DISCORD_TOKEN").expect(
        "Expected a token in the
    environment",
    );

    let database_url = env::var("DATABASE_URL")
        .expect("Expected `DATABASE_URL` in the environment");
    let redis_url =
        env::var("REDIS_URL").expect("Expected `REDIS_URL` in the environment");

    let http = Http::new_with_token(&token);

    let (owners, bot_id) = match http.get_current_application_info().await {
        Ok(info) => {
            let mut owners = HashSet::new();
            owners.insert(info.owner.id);

            (owners, info.id)
        },

        Err(e) => panic!("Could not access application info: {:?}", e),
    };

    let framework = StandardFramework::new()
        .configure(|c| {
            c.owners(owners)
                .dynamic_prefix(|ctx, msg| {
                    Box::pin(async move {
                        let type_map = ctx.data.read().await;
                        let db = type_map
                            .get::<Database>()
                            .expect("Expected Database in TypeMap")
                            .lock()
                            .await;

                        db.get_guild_prefix(msg.guild_id.unwrap()).ok()
                    })
                })
                .on_mention(Some(bot_id))
                .with_whitespace(true)
                .allow_dm(false)
        })
        .help(&HELP_CMD)
        .group(&METACMDS_GROUP)
        .group(&XPCMDS_GROUP)
        .normal_message(hooks::normal_message);

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .intents(
            GatewayIntents::GUILDS
                | GatewayIntents::GUILD_MEMBERS
                | GatewayIntents::GUILD_MESSAGES
                | GatewayIntents::GUILD_PRESENCES, // ugh
        )
        .await
        .expect("Err creating client");

    let redis = RedisCache::new(&redis_url);
    let db = Arc::new(Mutex::new(Database::new(&database_url, redis)));
    let msg_xp_timeout_cache = Arc::new(Mutex::new(LruCache::<
        (UserId, GuildId),
        i32,
    >::with_expiry_duration(
        Duration::from_secs(XP_TIMEOUT_SECS),
    )));

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<Database>(db.clone());
        data.insert::<MessageXPTimeoutCache>(msg_xp_timeout_cache.clone());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        shard_manager.lock().await.shutdown_all().await;
    });

    if let Err(e) = client.start_autosharded().await {
        error!("Client error: {:?}", e);
    }
}
