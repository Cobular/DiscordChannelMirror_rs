# Discord Channel Mirror rs

A drop-dead simple bot to mirror messages from one channel to another (inc. message content, profile pics, ) with a webhook. We use it for a quote wall on our server, but you could use this for any number of things.

## Getting Started

A docker image should be on docker hub at coverj715/discord_channel_mirror_rs. Just clone the latest version of that and run it with the config environment variables:

1. `DISCORD_TOKEN`: A Discord Bot Token
2. `SOURCE_CHANNEL_ID`: The ID of the channel the bot should copy all messages from. 
