extern crate discord;
#[macro_use]
extern crate lazy_static;
extern crate markov;
extern crate regex;

use discord::model::{Channel, ChannelId, ChannelType, Event, LiveServer, Message, MessageId,
                     PossibleServer, PublicChannel, RoleId, ServerId, UserId};
use discord::{Connection, Discord, GetMessages};

use regex::Regex;

use std::env;
use std::error::Error;

struct Config {
    auto_post_enabled: bool,
    pinging_enabled: bool,
    freq: u64,
    channel_id: ChannelId,
}

fn send_message(message: &str, discord: &Discord, channel_id: ChannelId) {
    if let Err(err) = discord.send_message(channel_id, message, "", false) {
        println!("ERROR: {}", err.description());
    }
}

fn send_info(message: &str, discord: &Discord, channel_id: ChannelId) {
    send_message(&format!("INFO: {}", message), discord, channel_id);
}

fn send_error(message: &str, discord: &Discord, channel_id: ChannelId) {
    send_message(&format!("ERROR: {}", message), discord, channel_id);
}

fn get_messages_in_channel(channel: &PublicChannel, discord: &Discord) -> Vec<Message> {
    let mut all_messages = Vec::new();
    let mut msg_id;

    let maybe_last_message =
        discord.get_message(channel.id, channel.last_message_id.unwrap_or(MessageId(0)));
    let last_message = match maybe_last_message {
        Ok(message) => message,
        Err(_) => return all_messages,
    };
    all_messages.push(last_message);
    msg_id = all_messages.last().unwrap().id;
    loop {
        let maybe_messages =
            discord.get_messages(channel.id, GetMessages::Before(msg_id), Some(100));
        match maybe_messages {
            Ok(messages) => {
                if messages.len() == 0 {
                    break;
                }
                all_messages.extend(messages);
                msg_id = all_messages.last().unwrap().id;
            }
            Err(_) => panic!("Error getting messages"),
        }
    }
    all_messages
}

fn get_messages_in_server(server: &LiveServer, discord: &Discord) -> Vec<Message> {
    server
        .channels
        .iter()
        .filter(|channel| channel.kind == ChannelType::Text)
        .map(|channel| get_messages_in_channel(channel, discord))
        .fold(vec![], |mut vec1, vec2| {
            vec1.extend(vec2);
            vec1
        })
}

fn is_convo_message(message: &Message) -> bool {
    !message.author.bot && !message.content.starts_with("/") && !message.content.starts_with("!")
        && !message.content.starts_with("?")
}

fn get_state_str(state: bool) -> &'static str {
    if state {
        "enabled"
    } else {
        "disabled"
    }
}

fn get_username(user_id: UserId, discord: &Discord, server_id: ServerId) -> String {
    match discord.get_member(server_id, user_id) {
        Ok(member) => member.nick.unwrap_or(member.user.name),
        Err(_) => String::from("INVALID-USERNAME"),
    }
}

fn str_to_user_id(str: &str) -> UserId {
    UserId(str.parse().unwrap_or(0))
}

fn remove_user_pings(message: &str, discord: &Discord, server_id: ServerId) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"<@!?(\d+)>").unwrap();
    }
    RE.replace_all(message, |captures: &regex::Captures| {
        let user_id = str_to_user_id(&captures[1]);
        let username = get_username(user_id, discord, server_id);
        format!("@{}", username)
    }).into_owned()
}

fn get_role_name(role_id: RoleId, server: &LiveServer) -> String {
    let maybe_role = server.roles.iter().find(|role| role.id == role_id);
    match maybe_role {
        Some(role) => role.name.clone(),
        None => String::from("INVALID-ROLE"),
    }
}

fn str_to_role_id(str: &str) -> RoleId {
    RoleId(str.parse().unwrap_or(0))
}

fn remove_role_pings(message: &str, server: &LiveServer) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"<@&(\d+)>").unwrap();
    }
    RE.replace_all(message, |captures: &regex::Captures| {
        let role_id = str_to_role_id(&captures[1]);
        let role_name = get_role_name(role_id, server);
        format!("@{}", role_name)
    }).into_owned()
}

fn remove_special_pings(message: &str) -> String {
    /*
     * Add zero-width unicode spaces to prevent pings; this may not work in the
     * future.
     */
    message
        .replace("@someone", "@\u{200B}someone")
        .replace("@everyone", "@\u{200B}everyone")
        .replace("@here", "@\u{200B}here")
}

fn remove_pings(message: &str, discord: &Discord, server: &LiveServer) -> String {
    remove_special_pings(&remove_user_pings(
        &remove_role_pings(message, server),
        discord,
        server.id,
    ))
}

fn send_nonsense(
    markov_chain: &markov::Chain<String>,
    discord: &Discord,
    server: &LiveServer,
    channel_id: ChannelId,
    pinging_enabled: bool,
) {
    let nonsense_with_pings = markov_chain.generate_str();
    let nonsense = if pinging_enabled {
        nonsense_with_pings
    } else {
        remove_pings(&nonsense_with_pings, discord, server)
    };
    send_message(&nonsense, discord, channel_id);
}

fn get_token() -> String {
    env::var("DISCORD_TOKEN").expect(
        "No Discord client token; Run this bot with the DISCORD_TOKEN \
         environment variable set",
    )
}

fn get_default_channel_id() -> ChannelId {
    ChannelId(
        env::var("DISCORD_CHANNEL_ID")
            .expect(
                "No default channel; Run this bot with the 
                    DISCORD_CHANNEL_ID environment variable set",
            )
            .parse()
            .expect("Invalid channel ID"),
    )
}

fn login() -> Discord {
    Discord::from_bot_token(&get_token()).expect("Login failed")
}

fn lookup_public_channel(channel_id: ChannelId, discord: &Discord) -> PublicChannel {
    match discord.get_channel(channel_id) {
        Ok(Channel::Public(channel)) => channel,
        Ok(_) => panic!(
            "Channel is either a DM or a group chat; It should be \
             a server"
        ),
        Err(_) => panic!("Invalid channel ID"),
    }
}

fn lookup_server(server_id: ServerId, connection: &mut Connection) -> LiveServer {
    loop {
        match connection.recv_event() {
            Ok(Event::ServerCreate(PossibleServer::Online(server))) => {
                if server.id == server_id {
                    return server;
                }
            }
            Ok(_) => {}
            Err(err) => panic!("Received error: {:?}", err),
        }
    }
}

fn main() {
    println!("Logging in");
    let discord = login();

    println!("Connecting bot");
    let (mut connection, _) = discord.connect().expect("Connection failed");
    println!("Connected");

    let mut config = Config {
        auto_post_enabled: true,
        pinging_enabled: true,
        freq: 1,
        channel_id: get_default_channel_id(),
    };
    let default_channel = lookup_public_channel(config.channel_id, &discord);

    println!("Looking up Discord server");
    let server = lookup_server(default_channel.server_id, &mut connection);

    println!("Retrieving server messages");
    let messages = get_messages_in_server(&server, &discord);
    let convo_messages: Vec<&Message> = messages
        .iter()
        .filter(|message| is_convo_message(&message))
        .collect();
    if convo_messages.len() == 0 {
        panic!("Server has no conversation messages");
    }

    println!("Populating markov chain");
    let mut markov_chain = markov::Chain::new();
    for message in convo_messages {
        markov_chain.feed_str(&message.content);
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
                    send_info(
                        "I don't listen to group chats",
                        &discord,
                        message.channel_id,
                    );
                    continue;
                }
                if let Ok(Channel::Private(_)) = channel {
                    send_info("I don't listen to DMs", &discord, message.channel_id);
                    continue;
                }
                if message.content.starts_with("!nonsense info") {
                    let info = &format!(
                        "Nonsense bot information:\n\
                         \n\
                         Automatic posting is {}\n\
                         Pinging is {}\n\
                         Post frequency = {}\n",
                        get_state_str(config.auto_post_enabled),
                        get_state_str(config.pinging_enabled),
                        config.freq
                    );
                    send_info(info, &discord, config.channel_id);
                } else if message.content.starts_with("!nonsense here") {
                    config.channel_id = message.channel_id;
                    send_info("Switched channels", &discord, config.channel_id);
                } else if message.content.starts_with("!nonsense on") {
                    config.auto_post_enabled = true;
                    send_info("Posting enabled", &discord, config.channel_id);
                } else if message.content.starts_with("!nonsense off") {
                    config.auto_post_enabled = false;
                    send_info("Posting disabled", &discord, config.channel_id);
                } else if message.content.starts_with("!nonsense ping on") {
                    config.pinging_enabled = true;
                    send_info("Pinging enabled", &discord, config.channel_id);
                } else if message.content.starts_with("!nonsense ping off") {
                    config.pinging_enabled = false;
                    send_info("Pinging disabled", &discord, config.channel_id);
                } else if message.content.starts_with("!nonsense freq") {
                    let maybe_third_field_val = message
                        .content
                        .split(' ')
                        .nth(2)
                        .unwrap_or("")
                        .parse::<u64>();
                    match maybe_third_field_val {
                        Ok(new_freq) if new_freq > 0 => {
                            config.freq = new_freq;
                            send_info(
                                &format!("Changed post frequency to {}", config.freq),
                                &discord,
                                config.channel_id,
                            );
                        }
                        Ok(_) => {
                            send_error(
                                "Invalid frequency (stop trying to \
                                 cause trouble :wink:)",
                                &discord,
                                config.channel_id,
                            );
                        }
                        Err(err) => {
                            send_error(err.description(), &discord, config.channel_id);
                        }
                    }
                } else if message.content.starts_with("!nonsense") {
                    send_nonsense(
                        &markov_chain,
                        &discord,
                        &server,
                        config.channel_id,
                        config.pinging_enabled,
                    );
                } else {
                    if is_convo_message(&message) {
                        markov_chain.feed_str(&message.content);
                    }
                    if message.id.0 % config.freq == 0 && config.auto_post_enabled {
                        send_nonsense(
                            &markov_chain,
                            &discord,
                            &server,
                            config.channel_id,
                            config.pinging_enabled,
                        );
                    }
                }
            }
            Ok(_) => {}
            Err(discord::Error::Closed(code, body)) => {
                panic!("Gateway closed on us with code {:?}: {}", code, body)
            }
            Err(err) => println!("ERROR: {}", err.description()),
        }
    }
}
