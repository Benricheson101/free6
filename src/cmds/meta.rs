use std::collections::HashSet;

use diesel::result::{DatabaseErrorKind, Error as DieselError};
use serenity::{
    framework::standard::{
        help_commands,
        macros::{command, help},
        Args,
        CommandGroup,
        CommandResult,
        HelpOptions,
    },
    model::prelude::*,
    prelude::*,
    utils::MessageBuilder,
};
use tokio::time::Instant;
use tracing::error;

use crate::util::db::Database;

#[command("ping")]
#[description = "Pong! See how long it takes the bot to respond"]
pub async fn ping_cmd(ctx: &Context, msg: &Message) -> CommandResult {
    let now = Instant::now();

    let mut m = msg.channel_id.say(&ctx.http, ":ping_pong:").await?;

    let elapsed = now.elapsed().as_millis();

    m.edit(&ctx, |e| {
        e.content(format!(":ping_pong: Pong! Message sent in {}ms", elapsed))
    })
    .await?;

    Ok(())
}

#[command("github")]
#[description = "Get a link to the GitHub repository"]
pub async fn github_cmd(ctx: &Context, msg: &Message) -> CommandResult {
    msg.channel_id
        .say(&ctx.http, "https://github.com/Benricheson101/free6")
        .await?;

    Ok(())
}

#[command("get_user")]
#[owners_only]
pub async fn get_user_cmd(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let db = data
        .get::<Database>()
        .expect("Expected `Database` in TypeMap")
        .lock()
        .await;

    let found = db.get_guild_user(msg.author.id, msg.guild_id.unwrap()).ok();

    match found {
        Some(d) => {
            msg.channel_id
                .say(&ctx.http, format!("```rs\n{:#?}```", d))
                .await?;
        },
        None => {
            msg.channel_id
                .say(&ctx.http, "You are not in the database")
                .await?;
        },
    }

    Ok(())
}

#[command("create_user")]
pub async fn create_user_cmd(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let db = data
        .get::<Database>()
        .expect("Expected `Database` in TypeMap")
        .lock()
        .await;

    let created = db.create_guild_user(msg.author.id, msg.guild_id.unwrap());

    match created {
        Ok(u) => {
            msg.channel_id
                .say(&ctx.http, format!("```rs\n{:#?}```", u))
                .await?;
        },
        Err(e) => {
            match &e {
                DieselError::DatabaseError(_kind, err) => {
                    msg.channel_id
                        .say(&ctx.http, format!("{:?}", *err))
                        .await?;
                },
                _ => (),
            }

            error!("{:#?}", e);
        },
    }

    Ok(())
}

#[command("get_all_users")]
#[owners_only]
pub async fn get_all_users_cmd(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let db = data
        .get::<Database>()
        .expect("Expected `Database` in TypeMap")
        .lock()
        .await;

    let found = db.get_guild_users(msg.guild_id.unwrap());

    match found {
        Ok(guilds) => {
            msg.channel_id
                .say(&ctx.http, format!("```rs\n{:#?}```", guilds))
                .await?;
        },
        Err(e) => {
            msg.channel_id
                .say(&ctx.http, format!("Error!\n```rs\n{:#?}```", e))
                .await?;
        },
    }

    Ok(())
}

#[help]
#[embed_success_colour = "#a97ccc"]
#[individual_command_tip = "To learn more about a command, pass its name as an argument"]
#[strikethrough_commands_tip_in_guild = ""]
async fn help_cmd(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(
        ctx,
        msg,
        args,
        help_options,
        groups,
        owners,
    )
    .await;

    Ok(())
}
