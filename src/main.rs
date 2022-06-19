mod cmd;
mod config;
mod handler;
mod state;

use state::State;

use std::collections::HashMap;

#[macro_use]
extern crate lazy_static;
use regex::Regex;
use serenity::{
    framework::standard::{macros::hook, CommandResult, StandardFramework},
    futures::{prelude::*, stream},
    http::Http,
    model::{channel::Message, id::ChannelId, prelude::*},
    prelude::*,
    utils::Color,
    Result,
};

async fn get_username(user_id: UserId, http: &Http, guild_id: GuildId) -> String {
    let member = http.get_member(guild_id.0, user_id.0).await;
    match member {
        Ok(member) => member.nick.unwrap_or(member.user.name),
        Err(_) => String::from("INVALID-USERNAME"),
    }
}

fn str_to_user_id(str: &str) -> UserId {
    UserId(str.parse().unwrap_or(0))
}

async fn remove_user_pings(message: &str, http: &Http, guild_id: GuildId) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"<@!?(\d+)>").unwrap();
    }
    let user_ids = RE
        .captures_iter(message)
        .map(|captures| str_to_user_id(&captures[1]))
        .collect::<Vec<UserId>>();
    let usernames = stream::iter(user_ids)
        .then(|user_id| async move {
            let name = get_username(user_id, http, guild_id).await;
            (user_id, name)
        })
        .collect::<HashMap<UserId, String>>()
        .await;
    RE.replace_all(message, |captures: &regex::Captures| {
        let user_id = str_to_user_id(&captures[1]);
        let username = &usernames[&user_id];
        format!("@{}", username)
    })
    .into_owned()
}

async fn get_role_name(role_id: RoleId, guild_id: GuildId, http: &Http) -> String {
    let roles = guild_id
        .roles(http)
        .await
        .expect("Failed getting list of roles in guild");
    let maybe_role = roles.values().find(|role| role.id == role_id);
    match maybe_role {
        Some(role) => role.name.clone(),
        None => String::from("INVALID-ROLE"),
    }
}

fn str_to_role_id(str: &str) -> RoleId {
    RoleId(str.parse().unwrap_or(0))
}

async fn remove_role_pings(message: &str, guild_id: GuildId, http: &Http) -> String {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"<@&(\d+)>").unwrap();
    }
    let role_ids = RE
        .captures_iter(message)
        .map(|captures| str_to_role_id(&captures[1]))
        .collect::<Vec<RoleId>>();
    let role_names = stream::iter(role_ids)
        .then(|role_id| async move {
            let role_name = get_role_name(role_id, guild_id, http).await;
            (role_id, role_name)
        })
        .collect::<HashMap<RoleId, String>>()
        .await;
    RE.replace_all(message, |captures: &regex::Captures| {
        let role_id = str_to_role_id(&captures[1]);
        let role_name = &role_names[&role_id];
        format!("@{}", role_name)
    })
    .into_owned()
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

async fn remove_pings(message: &str, http: &Http, guild_id: GuildId) -> String {
    let message = remove_role_pings(message, guild_id, http).await;
    let message = remove_user_pings(&message, http, guild_id).await;
    remove_special_pings(&message)
}

async fn send_nonsense(ctx: &Context, channel_id: ChannelId, guild_id: GuildId) -> Result<()> {
    let http = &ctx.http;
    let data = ctx.data.read().await;
    let state = get_state(&data).await;
    let guilds = state.guilds.read().await;
    let guild_state = &guilds[&guild_id].read().await;
    let markov_chain = &guild_state.markov_chain;
    if markov_chain.is_empty() {
        return Ok(());
    }
    let nonsense_with_pings = markov_chain.generate_str();
    let nonsense = if guild_state.config.pinging_enabled {
        nonsense_with_pings
    } else {
        remove_pings(&nonsense_with_pings, http, guild_id).await
    };
    channel_id.say(http, nonsense).await?;
    Ok(())
}

#[hook]
async fn before_command_hook(_ctx: &Context, msg: &Message, cmd: &str) -> bool {
    println!("Got command '{}' by user '{}'", cmd, msg.author.name);
    true
}

async fn send_error(http: &Http, channel_id: ChannelId, text: impl ToString) {
    let res = channel_id
        .send_message(http, |m| {
            m.embed(|e| e.color(Color::RED).title("Error").description(text))
        })
        .await;
    if let Err(err) = res {
        println!("Error {err}");
    }
}

#[hook]
async fn after_command_hook(ctx: &Context, msg: &Message, _cmd: &str, res: CommandResult) {
    // If `res` holds an error, reply to a message with the error
    if let Err(err) = res {
        println!("Error {err}");
        send_error(&ctx.http, msg.channel_id, err).await;
    }
}

#[hook]
async fn unrecognised_command_hook(ctx: &Context, msg: &Message, cmd: &str) {
    send_error(
        &ctx.http,
        msg.channel_id,
        format!("Command '{cmd}' unrecognized"),
    )
    .await;
}

async fn get_state(data: &TypeMap) -> &State {
    data.get::<State>().expect("No state in context")
}

async fn create_client(token: &str, prefix: &str) -> Result<Client> {
    let framework = StandardFramework::new()
        .configure(|c| c.prefix(prefix))
        .group(&cmd::GENERAL_GROUP)
        .help(&cmd::HELP)
        .before(before_command_hook)
        .after(after_command_hook)
        .unrecognised_command(unrecognised_command_hook);
    Client::builder(token, GatewayIntents::all())
        .event_handler(handler::Handler)
        .framework(framework)
        .type_map_insert::<State>(Default::default())
        .await
}

#[tokio::main]
async fn main() {
    println!("Loading environment variables...");
    let token = std::env::var("NONSENSE_TOKEN").expect(
        "No Discord client token; Run this bot with the NONSENSE_TOKEN \
         environment variable set",
    );
    let prefix = std::env::var("NONSENSE_PREFIX").expect(
        "No bot command prefix; Run this bot with the NONSENSE_PREFIX \
         environment variable set",
    );

    println!("Creating client...");
    let mut client = create_client(&token, &prefix)
        .await
        .expect("Failed creating client");

    println!("Starting client...");
    client.start().await.expect("Error running client");
}
