extern crate discord;
extern crate markov;

use discord::model::{Channel, ChannelId, Event, MessageId, ReadyEvent};
use discord::{Connection, Discord, GetMessages, State};

use std::env;
use std::error::Error;

fn send_message(message : &str, discord : &Discord, channel_id : ChannelId) {
    if let Err(err) = discord.send_message(channel_id, message, "", false) {
        println!("ERROR: {}", err.description());
    }
}

fn send_info(message : &str, discord : &Discord, channel_id : ChannelId) {
    send_message(&format!("INFO: {}", message), discord, channel_id);
}

fn send_error(message : &str, discord : &Discord, channel_id : ChannelId) {
    send_message(&format!("ERROR: {}", message), discord, channel_id);
}

fn main() {
    println!("Logging in");
    let discord = &Discord::from_bot_token(
        &env::var("DISCORD_CLIENT_TOKEN")
        .expect("No Discord client token; Run this bot with the \
            DISCORD_CLIENT_TOKEN environment variable set"),
    ).expect("Login failed");

    println!("Connecting bot");
    let (mut connection, ready) = discord.connect().expect("Connection failed");
    println!("Connected");

    let current_user = discord.get_current_user()
        .expect("Failed to get current user");

    let mut posting_enabled = true;
    let mut freq = 1;
    let mut channel_id = ChannelId(
        env::var("DISCORD_CHANNEL_ID")
        .expect("No default channel; Run this bot with the DISCORD_CHANNEL_ID \
                environment variable set")
        .parse()
        .expect("Invalid channel ID")
    );
    let default_channel = match discord.get_channel(channel_id) {
        Ok(Channel::Public(channel)) => channel,
        Ok(_) => panic!("Channel is either a DM or a group chat; It should be \
            a server"),
        Err(err) => panic!("{}", err.description())
    };

    let state = State::new(ready);
    println!("Bot has access to {} servers", state.servers().len());
    
    println!("Looking up Discord server");
    let server = state
        .find_server(default_channel.server_id)
        .expect("Cannot find server from specified channel");

    println!("Retrieving server messages");
    let messages = server
        .channels
        .iter()
        .map(
            |channel| discord.get_messages(channel.id, GetMessages::MostRecent,
                Some(u64::max_value()))
            .expect("Failed to get messages")
        )
        .fold(vec!(), |mut vec1, vec2| {
            vec1.extend(vec2);
            vec1
        });

    println!("Populating markov chain");
    let mut markov_chain = markov::Chain::new();
    for message in messages {
        markov_chain.feed_str(&message.content);
    }

    loop {
        match connection.recv_event() {
            Ok(Event::MessageCreate(message)) => {
                // Ignores the bot's own messages
                if message.author.id == current_user.id {
                    continue;
                }
                let channel = discord.get_channel(message.channel_id);
                if let Ok(Channel::Group(_)) = channel {
                    send_info("I don't listen to group chats", discord,
                        message.channel_id);
                    continue;
                }
                if let Ok(Channel::Private(_)) = channel {
                    send_info("I don't listen to DMs", discord,
                        message.channel_id);
                    continue;
                }
                let third_field_val = message.content
                    .split(' ')
                    .nth(2)
                    .unwrap_or("")
                    .parse::<u64>();
                let MessageId(message_id) = message.id;
                if message.content.starts_with("!nonsense info") {
                    let posting_state =
                        if posting_enabled {
                            "enabled"
                        } else {
                            "disabled"
                        };
                    let info = &format!(
                        "Nonsense bot information:\n\
                        \n\
                        Posting is {}\n\
                        Post frequency = {}\n",
                        posting_state, freq
                    );
                    send_info(info, discord, channel_id);
                } else if message.content.starts_with("!nonsense here") {
                    channel_id = message.channel_id;
                    send_info("Switched channels", discord, channel_id);
                } else if message.content.starts_with("!nonsense on") {
                    posting_enabled = true;
                    send_info("Posting enabled", discord, channel_id);
                } else if message.content.starts_with("!nonsense off") {
                    posting_enabled = false;
                    send_info("Posting disabled", discord, channel_id);
                } else if message.content.starts_with("!nonsense freq") {
                    match third_field_val {
                        Ok(new_freq) => {
                            freq = new_freq;
                            send_info(&format!("Changed post frequency to {}",
                                freq), discord, channel_id);
                        }
                        Err(err) => {
                            send_error(err.description(), discord, channel_id);
                        }
                    }
                // TODO: More checking
                } else if message.content.starts_with("!nonsense") {
                    let wisdom = &markov_chain.generate_str();
                    send_message(wisdom, discord, channel_id);
                } else {
                    markov_chain.feed_str(&message.content);
                    let wisdom = &markov_chain.generate_str();
                    if message_id % freq == 0 && posting_enabled {
                        send_message(wisdom, discord, channel_id);
                    }
                }
            }
            Ok(_) => {}
            Err(discord::Error::Closed(code, body)) =>
                panic!("Gateway closed on us with code {:?}: {}", code, body),
            Err(err) => panic!("Received error: {:?}", err)
        }
    }
}
