use crate::get_state;

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
    let data = ctx.data.read().await;
    let state = get_state(&data).await;
    let guilds = state.guilds.read().await;
    let username = ctx.http.get_current_user().await?.name;
    let config = guilds[&guild_id].read().await.config;
    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title(format!("{username} information"))
                    .description(config)
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
    let data = ctx.data.read().await;
    let state = get_state(&data).await;
    let guilds = state.guilds.read().await;
    guilds[&guild_id].write().await.config.auto_post_enabled = true;
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
    let data = ctx.data.read().await;
    let state = get_state(&data).await;
    let guilds = state.guilds.read().await;
    guilds[&guild_id].write().await.config.auto_post_enabled = false;
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
    let data = ctx.data.read().await;
    let state = get_state(&data).await;
    let guilds = state.guilds.read().await;
    guilds[&guild_id].write().await.config.pinging_enabled = true;
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
    let data = ctx.data.write().await;
    let state = get_state(&data).await;
    let guilds = state.guilds.read().await;
    guilds[&guild_id].write().await.config.pinging_enabled = false;
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
    if new_freq > 0 {
        return Err("Invalid frequency".into());
    }
    let data = ctx.data.read().await;
    let state = get_state(&data).await;
    let guilds = state.guilds.read().await;
    let mut config = guilds[&guild_id].write().await.config;
    config.freq = new_freq;
    let response = format!("Changed post frequency to {}", config.freq);
    msg.channel_id
        .send_message(&ctx.http, |m| m.embed(|e| e.description(response)))
        .await?;
    Ok(())
}
