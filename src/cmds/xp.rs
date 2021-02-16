use std::collections::HashSet;

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

use crate::util::{
    db::Database,
    xp::{xp_to_lvl, xp_to_next_lvl},
};

#[command("set_xp")]
#[owners_only]
pub async fn set_xp_cmd(
    ctx: &Context,
    msg: &Message,
    mut args: Args,
) -> CommandResult {
    if let Ok(n) = args.single::<u32>() {
        let data = ctx.data.read().await;
        let db = data
            .get::<Database>()
            .expect("Expected `Database` in TypeMap")
            .lock()
            .await;

        let saved = db
            .set_guild_user_xp(msg.author.id, msg.guild_id.unwrap(), n as i32)
            .unwrap();

        msg.channel_id
            .say(&ctx.http, format!("```rs\n{:#?}```", saved))
            .await?;
    } else {
        msg.channel_id.say(&ctx.http, "Invalid argument").await?;
    }

    Ok(())
}

#[command("rank")]
#[aliases("level", "levels", "ranking")]
pub async fn rank_cmd(ctx: &Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read().await;
    let db = data
        .get::<Database>()
        .expect("Expected `Database` in TypeMap")
        .lock()
        .await;

    let m = match db.get_guild_user(msg.author.id, msg.guild_id.unwrap()) {
        Ok(u) => format!("You are level {} ({} xp)", xp_to_lvl(u.xp), u.xp),
        Err(_) => "You are not in the database. Run the `~create_user` command and try again".to_string(),
    };

    msg.channel_id.say(&ctx.http, &m).await?;

    Ok(())
}
