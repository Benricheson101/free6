use rand::Rng;
use serenity::{
    framework::standard::macros::hook,
    model::prelude::*,
    prelude::*,
};

use crate::{
    util::db::Database,
    MessageXPTimeoutCache,
    MAX_MESSAGE_XP,
    MIN_MESSAGE_XP,
};

#[hook]
pub async fn normal_message(ctx: &Context, msg: &Message) {
    println!("Received normal message: {}", &msg.content);

    let data = ctx.data.read().await;

    let mut timeout_cache = data
        .get::<MessageXPTimeoutCache>()
        .expect("Expected `MessageXPTimeoutCache` in TypeMap")
        .lock()
        .await;

    let cache_key = (msg.author.id, msg.guild_id.unwrap());

    if timeout_cache.peek(&cache_key).is_some() {
        println!("{} is on cooldown", msg.author.name);
        return;
    }

    let db = data
        .get::<Database>()
        .expect("expected `database` in typemap")
        .lock()
        .await;

    let xp_to_grant: i32 =
        rand::thread_rng().gen_range(MIN_MESSAGE_XP..MAX_MESSAGE_XP);

    if let Ok(saved) =
        db.add_guild_user_xp(msg.author.id, msg.guild_id.unwrap(), xp_to_grant)
    {
        let prev_amt = saved.xp - xp_to_grant;
        let curr_amt = saved.xp;

        let prev_lvl = crate::util::xp::xp_to_lvl(prev_amt);
        let curr_lvl = crate::util::xp::xp_to_lvl(curr_amt);

        if prev_lvl != curr_lvl {
            msg.channel_id
                .say(&ctx.http, format!("Level {}", curr_lvl))
                .await
                .ok();
        }
    }

    timeout_cache.insert(cache_key, 0);
}
