use std::{env, error::Error, fmt::Debug};

use futures::StreamExt;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use twilight_gateway::{Cluster, Event, EventType, EventTypeFlags, Intents};

#[derive(Debug, Deserialize, Serialize)]
struct GatewayPayload {
    op: u32,
    d: serde_json::Value,
    s: Option<u64>,
    t: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let redis_client = redis::Client::open("redis://127.0.0.1").unwrap();
    let mut redis_conn = redis_client.get_async_connection().await?;

    let token =
        env::var("DISCORD_TOKEN").expect("Unable to read `DISCORD_TOKEN`");
    let intents = Intents::GUILD_MESSAGES | Intents::GUILDS;

    let cluster = Cluster::new(token, intents).await?;

    let mut events =
        cluster.some_events(EventTypeFlags::from(EventType::ShardPayload));

    let cluster_spawn = cluster.clone();

    tokio::spawn(async move {
        cluster_spawn.up().await;
    });

    while let Some((_shard_id, event)) = events.next().await {
        if let Event::ShardPayload(shard) = event {
            let payload: GatewayPayload = serde_json::from_slice(&shard.bytes)?;
            handle_event(&mut redis_conn, &payload).await?;
            if let Some(event_name) = &payload.t {
                println!("{}", &event_name);
                publish(
                    &mut redis_conn,
                    event_name,
                    &serde_json::to_string(&payload.d)?,
                )
                .await?;
            }
        }
    }

    Ok(())
}

async fn handle_event(
    conn: &mut redis::aio::Connection,
    event: &GatewayPayload,
) -> Result<(), Box<dyn Error>> {
    if let Some(event_name) = &event.t {
        match event_name.as_str() {
            "GUILD_CREATE" => {
                if let Some(guild_id) = event.d["id"].as_str() {
                    conn.hset(
                        "guilds",
                        guild_id,
                        serde_json::to_string(&event.d)?,
                    )
                    .await?;
                }
            },

            "MESSAGE_CREATE" if !&event.d["webhook_id"].is_null() => {
                if let Some(user_id) = event.d["author"]["id"].as_str() {
                    conn.hset(
                        "users",
                        user_id,
                        serde_json::to_string(&event.d["author"])?,
                    )
                    .await?;
                }
            },

            &_ => (),
        }
    }

    Ok(())
}

async fn publish(
    conn: &mut redis::aio::Connection,
    event_name: &String,
    data: &String,
) -> Result<(), Box<dyn Error>> {
    conn.publish(format!("gateway:{}", event_name), data)
        .await?;
    Ok(())
}
