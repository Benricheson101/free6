mod cmds;

use std::{collections::HashSet, env, sync::Arc};

use cmds::meta::*;
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

#[group]
#[commands(ping_cmd, github_cmd)]
#[description = "Meta commands, idk, nothing too special here"]
struct Meta;

pub struct ShardManagerContainer;

impl TypeMapKey for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
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

    let token =
        env::var("DISCORD_TOKEN").expect("Expected a token in the environment");

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
        })
        .help(&HELP_CMD)
        .group(&META_GROUP);

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<ShardManagerContainer>(client.shard_manager.clone());
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
