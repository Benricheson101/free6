#![allow(dead_code, unused_imports)]

mod cmds;
pub mod models;
pub mod schema;
pub mod util;

use std::{collections::HashSet, env, sync::Arc};

#[macro_use]
extern crate diesel;

use cmds::{meta::*, xp::*};
use dotenv::dotenv;
use serenity::{
    async_trait,
    client::{bridge::gateway::ShardManager, EventHandler},
    framework::{standard::macros::group, StandardFramework},
    http::Http,
    model::{event::ResumedEvent, gateway::Ready},
    prelude::*,
};
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use util::db::Database;

#[group("Meta")]
#[commands(
    ping_cmd,
    github_cmd,
    get_user_cmd,
    create_user_cmd,
    get_all_users_cmd
)]
#[description = "Meta commands, idk, nothing too special here"]
struct MetaCmds;

#[group("XP")]
#[commands(set_xp_cmd, rank_cmd)]
#[description = "Commands related to the XP leveling system"]
struct XpCmds;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

impl TypeMapKey for Database {
    type Value = Arc<Mutex<Database>>;
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

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to start the logger");

    let token = env::var("DISCORD_TOKEN").expect(
        "Expected a token in the
    environment",
    );

    let database_url = env::var("DATABASE_URL")
        .expect("Expected a database url in the environment");

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
                .prefix("~")
                .on_mention(Some(bot_id))
                .with_whitespace(true)
                .allow_dm(false)
        })
        .help(&HELP_CMD)
        .group(&METACMDS_GROUP)
        .group(&XPCMDS_GROUP);

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    let db = Arc::new(Mutex::new(Database::new(&database_url)));

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
        data.insert::<Database>(db.clone());
    }

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        println!("[ctrl+c] before await");
        tokio::signal::ctrl_c()
            .await
            .expect("Could not register ctrl+c handler");
        println!("[ctrl+c] before shutdown");
        shard_manager.lock().await.shutdown_all().await;

        println!("[ctrl+c] after shutdown");
    });

    if let Err(e) = client.start_autosharded().await {
        error!("Client error: {:?}", e);
    }

    // let db = Database::new(&database_url);

    // // db.create_user(255834596766253057, 579466138992508928);
    // db.get_guild_user(255834596766253057, 579466138992508928)?;

    // Ok(())
}
