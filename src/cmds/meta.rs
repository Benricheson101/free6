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
};
use tokio::time::Instant;

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
    msg.channel_id.say(&ctx.http, "https://github.com/Benricheson101/free6").await?;

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
