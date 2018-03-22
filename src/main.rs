extern crate discord;
extern crate markov;

use discord::model::{
    Channel, ChannelId, ChannelType, Event, LiveServer, Message, MessageId,
    PossibleServer, PublicChannel
};
use discord::{Discord, GetMessages};

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

fn get_all_messages_in_channel(channel : &PublicChannel, discord : &Discord)
        -> Vec<Message> {
    let mut all_messages = Vec::new();
    let mut msg_id;

    let maybe_last_message = discord.get_message(channel.id,
            channel.last_message_id.unwrap_or(MessageId(0)));
    let last_message = match maybe_last_message {
        Ok(message) => message,
        Err(_) => return all_messages
    };
    all_messages.push(last_message);
    msg_id = all_messages.last().unwrap().id;
    loop {
        let maybe_messages = discord.get_messages(channel.id,
                GetMessages::Before(msg_id), Some(100));
        match maybe_messages {
            Ok(messages) => {
                if messages.len() == 0 {
                    break;
                }
                all_messages.extend(messages);
                msg_id = all_messages.last().unwrap().id;
            }
            Err(_) => panic!("Error getting messages")
        }
    }
    all_messages
}

fn get_all_messages_in_server(server : &LiveServer, discord : &Discord)
        -> Vec<Message> {
    server
        .channels
        .iter()
        .filter(|channel| channel.kind == ChannelType::Text)
        .map(|channel| get_all_messages_in_channel(channel, discord))
        .fold(vec!(), |mut vec1, vec2| {
            vec1.extend(vec2);
            vec1
        })
}

fn should_notice_message(message : &Message) -> bool {
    !message.author.bot &&
        !message.content.starts_with("/") &&
        !message.content.starts_with("!") &&
        !message.content.starts_with("?")
}

fn main() {
    println!("Logging in");
    let discord = &Discord::from_bot_token(
        &env::var("DISCORD_TOKEN")
        .expect("No Discord client token; Run this bot with the \
                DISCORD_TOKEN environment variable set"),
    ).expect("Login failed");

    println!("Connecting bot");
    let (mut connection, _) = discord.connect().expect("Connection failed");
    println!("Connected");

    let mut auto_post_enabled = true;
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
        Err(_) => panic!("Invalid channel ID")
    };

    println!("Looking up Discord server");
    let server;
    loop {
        match connection.recv_event() {
            Ok(Event::ServerCreate(PossibleServer::Online(srv))) => {
                if srv.id == default_channel.server_id {
                    server = srv;
                    println!("Found server");
                    break;
                }
            }
            Ok(_) => {}
            Err(err) => panic!("Received error: {:?}", err)
        }
    }

    println!("Retrieving server messages");
    let messages = get_all_messages_in_server(&server, &discord);

    println!("Populating markov chain");
    let mut markov_chain = markov::Chain::new();
    for message in messages {
        if should_notice_message(&message) {
            markov_chain.feed_str(&message.content);
        }
    }

    println!("Waiting for new messages");
    loop {
        match connection.recv_event() {
            Ok(Event::MessageCreate(message)) => {
                if message.author.bot {
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
                if message.content.starts_with("!nonsense info") {
                    let auto_post_state =
                        if auto_post_enabled {
                            "enabled"
                        } else {
                            "disabled"
                        };
                    let info = &format!(
                        "Nonsense bot information:\n\
                        \n\
                        Automatic posting is {}\n\
                        Post frequency = {}\n",
                        auto_post_state, freq
                    );
                    send_info(info, discord, channel_id);
                } else if message.content.starts_with("!nonsense here") {
                    channel_id = message.channel_id;
                    send_info("Switched channels", discord, channel_id);
                } else if message.content.starts_with("!nonsense on") {
                    auto_post_enabled = true;
                    send_info("Posting enabled", discord, channel_id);
                } else if message.content.starts_with("!nonsense off") {
                    auto_post_enabled = false;
                    send_info("Posting disabled", discord, channel_id);
                } else if message.content.starts_with("!nonsense freq") {
                    let maybe_third_field_val = message.content
                        .split(' ')
                        .nth(2)
                        .unwrap_or("")
                        .parse::<u64>();
                    match maybe_third_field_val {
                        Ok(new_freq) if new_freq > 0 => {
                            freq = new_freq;
                            send_info(&format!("Changed post frequency to {}",
                                    freq), discord, channel_id);
                        }
                        Ok(_) => {
                            send_error("Invalid frequency (stop trying to \
                                cause trouble :wink:)", discord, channel_id);
                        }
                        Err(err) => {
                            send_error(err.description(), discord, channel_id);
                        }
                    }
                } else if message.content.starts_with("!nonsense") {
                    let wisdom = &markov_chain.generate_str();
                    send_message(wisdom, discord, channel_id);
                } else {
                    if should_notice_message(&message) {
                        markov_chain.feed_str(&message.content);
                    }
                    if message.id.0 % freq == 0 && auto_post_enabled {
                        let wisdom = &markov_chain.generate_str();
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
