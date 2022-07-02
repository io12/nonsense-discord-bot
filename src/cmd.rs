use crate::get_guild_state;

use std::collections::HashSet;

use serenity::{
    framework::standard::{
        help_commands,
        macros::{command, group, help},
        Args, CommandGroup, CommandResult, HelpOptions,
    },
    model::prelude::*,
    prelude::*,
};

#[group]
#[commands(info, on, off, ping_on, ping_off, freq)]
pub struct General;

#[help]
#[lacking_role(strike)]
#[lacking_ownership(strike)]
#[lacking_permissions(strike)]
#[lacking_conditions(strike)]
#[wrong_channel(strike)]
async fn help(
    ctx: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(ctx, msg, args, help_options, groups, owners).await?;
    Ok(())
}

#[command]
#[description = "Post information about the bot's state"]
#[only_in(guilds)]
async fn info(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let username = ctx.http.get_current_user().await?.name;
    let guild_state_lock = get_guild_state(ctx, guild_id).await;
    let guild_state = guild_state_lock.read().await;
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("{username} information"))
                    .description(guild_state.config)
            })
        })
        .await?;
    Ok(())
}

#[command]
#[description = "Enable automatic posting"]
#[only_in(guilds)]
#[required_permissions(ADMINISTRATOR)]
async fn on(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild_state_lock = get_guild_state(ctx, guild_id).await;
    let mut guild_state = guild_state_lock.write().await;
    guild_state.config.auto_post_enabled = true;
    msg.channel_id
        .send_message(&ctx.http, |m| m.embed(|e| e.description("Posting enabled")))
        .await?;
    Ok(())
}

#[command]
#[description = "Disable automatic posting"]
#[only_in(guilds)]
#[required_permissions(ADMINISTRATOR)]
async fn off(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild_state_lock = get_guild_state(ctx, guild_id).await;
    let mut guild_state = guild_state_lock.write().await;
    guild_state.config.auto_post_enabled = false;
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| e.description("Posting disabled"))
        })
        .await?;
    Ok(())
}

#[command]
#[description = "Enable pinging"]
#[only_in(guilds)]
#[required_permissions(ADMINISTRATOR)]
async fn ping_on(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild_state_lock = get_guild_state(ctx, guild_id).await;
    let mut guild_state = guild_state_lock.write().await;
    guild_state.config.pinging_enabled = true;
    msg.channel_id
        .send_message(&ctx.http, |m| m.embed(|e| e.description("Pinging enabled")))
        .await?;
    Ok(())
}

#[command]
#[description = "Disable pinging"]
#[only_in(guilds)]
#[required_permissions(ADMINISTRATOR)]
async fn ping_off(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let guild_state_lock = get_guild_state(ctx, guild_id).await;
    let mut guild_state = guild_state_lock.write().await;
    guild_state.config.pinging_enabled = false;
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| e.description("Pinging disabled"))
        })
        .await?;
    Ok(())
}

#[command]
#[description = "Set `freq`, where the bot posts after about every `freq` posts"]
#[only_in(guilds)]
#[required_permissions(ADMINISTRATOR)]
async fn freq(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();
    let new_freq = args.parse::<u64>()?;
    if new_freq == 0 {
        return Err("Zero is not a valid frequency".into());
    }
    let guild_state_lock = get_guild_state(ctx, guild_id).await;
    let mut guild_state = guild_state_lock.write().await;
    guild_state.config.freq = new_freq;
    let response = format!("Changed post frequency to {new_freq}");
    msg.channel_id
        .send_message(&ctx.http, |m| m.embed(|e| e.description(response)))
        .await?;
    Ok(())
}
