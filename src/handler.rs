use crate::get_state;
use crate::is_convo_message;

use std::sync::Arc;

use serenity::async_trait;
use serenity::futures::future;
use serenity::futures::prelude::*;
use serenity::http::Http;
use serenity::model::channel::Message;
use serenity::model::prelude::*;
use serenity::prelude::*;

async fn get_messages_in_guild(guild: &Guild, http: &Http) -> Vec<Message> {
    let messages = stream::iter(
        guild
            .channels
            .values()
            // Only consider guild channels
            .filter_map(|channel| {
                if let Channel::Guild(guild_channel) = channel {
                    Some(guild_channel.clone())
                } else {
                    None
                }
            })
            // Only consider text channels
            .filter(|guild_channel| guild_channel.kind == ChannelType::Text),
    )
    // Get messages in channels
    .flat_map(|guild_channel| {
        guild_channel
            .id
            .messages_iter(http)
            .filter_map(|msg| -> future::Ready<Option<Message>> { future::ready(msg.ok()) })
    })
    .collect::<Vec<Message>>()
    .await;
    // Filter ignored messages
    messages.into_iter().filter(is_convo_message).collect()
}

/// Serenity handler for bot. This implements `EventHandler` to process all the
/// bot events.
pub struct Handler;

/// Implementation of event handler
#[async_trait]
impl EventHandler for Handler {
    /// Print a log message when the bot is ready
    async fn ready(&self, _: Context, ready: Ready) {
        println!(
            "{} is ready! Waiting to connect to a guild...",
            ready.user.name
        );
    }

    async fn guild_create(&self, ctx: Context, guild: Guild, _: bool) {
        let http = &ctx.http;
        let data = ctx.data.read().await;
        let state = get_state(&data);
        let guild_state = {
            let mut guilds = state.guilds.write().await;
            Arc::clone(guilds.entry(guild.id).or_default())
        };
        let mut guild_state = guild_state.write().await;
        println!("Connected to guild {}. Loading messages...", guild.id);
        for message in get_messages_in_guild(&guild, http).await {
            guild_state.markov_chain.feed_str(&message.content);
        }

        println!("Waiting for new messages in {}...", guild.id);
    }
}
