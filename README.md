# Wisdom Discord bot

## Setup

Install Rust. On Arch Linux, do `sudo pacman -S rust`.

Run with `DISCORD_TOKEN=... DISCORD_CHANNEL_ID=... cargo run`.

If the above command fails with an error about OpenSSL, run `cargo clean && OPENSSL_INCLUDE_DIR=/usr/include/openssl-1.0 OPENSSL_LIB_DIR=/usr/lib/openssl-1.0 cargo build` and run the above command again.

## Commands

`!wisdom info`: Post information about the bot's state

`!wisdom here`: Move the bot to the channel this command was posted in

`!wisdom on`: Enable automatic posting

`!wisdom off`: Disable automatic posting

`!wisdom ping on`: Enable pinging

`!wisdom ping off`: Disable pinging

`!wisdom freq <int>`: Set `freq`, where the bot posts after about every `freq` posts

`!wisdom`: Generate and post a message
