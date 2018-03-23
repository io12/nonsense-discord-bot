# Nonsense bot

## Setup

Install Rust. On Arch Linux, do `sudo pacman -S rust`.

Run with `DISCORD_TOKEN=... DISCORD_CHANNEL_ID=... cargo run`.

If the above command fails with an error about OpenSSL, run `cargo clean && OPENSSL_INCLUDE_DIR=/usr/include/openssl-1.0 OPENSSL_LIB_DIR=/usr/lib/openssl-1.0 cargo build` and run the above command again.

## Commands

`!nonsense info`: Post information about the bot's state

`!nonsense here`: Move the bot to the channel this command was posted in

`!nonsense on`: Enable automatic posting

`!nonsense off`: Disable automatic posting

`!nonsense ping on`: Enable pinging

`!nonsense ping off`: Disable pinging

`!nonsense freq <int>`: Set `freq`, where the bot posts after about every `freq` posts

`!nonsense`: Generate and post a message
