extern crate dotenv;

use dotenv::dotenv;
use lazy_static::lazy_static;
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler, bridge::gateway::GatewayIntents},
    http::AttachmentType::Image,
    model::{channel::Message, id::ChannelId},
};
use std::env;
use std::time::Instant;

lazy_static! {
    static ref CHANNEL_ID: ChannelId = ChannelId(
        env::var("SOURCE_CHANNEL_ID")
            .expect("SOURCE_CHANNEL_ID")
            .parse()
            .unwrap(),
    );
    static ref TARGET_CHANNEL_ID: u64 = env::var("TARGET_CHANNEL_ID")
        .expect("TARGET_CHANNEL_ID")
        .parse()
        .unwrap();
    static ref TARGET_CHANNEL_WEBHOOK_TOKEN: String =
        env::var("TARGET_CHANNEL_WEBHOOK_TOKEN").expect("TARGET_CHANNEL_WEBHOOK_TOKEN");
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, incoming_message: Message) {
        let start = Instant::now();
        if incoming_message.channel_id == *CHANNEL_ID {
            let webhook = ctx
                .http
                .get_webhook_with_token(*TARGET_CHANNEL_ID, &TARGET_CHANNEL_WEBHOOK_TOKEN)
                .await
                .unwrap();

            // Get username or
            let username: String = match incoming_message.author_nick(&ctx.http).await {
                Some(nick) => nick,
                None => incoming_message.author.name.clone(),
            };

            let Message {
                attachments,
                content,
                author,
                ..
            } = incoming_message;

            let file_urls = attachments.iter().map(|attachment| Image(&attachment.url));

            let webhook_res = webhook
                .execute(&ctx.http, false, |w| {
                    w.content(content)
                        .avatar_url(author.avatar_url().unwrap_or_else(|| {
                            String::from("https://doc.rust-lang.org/rust-logo1.58.0.png")
                        }))
                        .username(username)
                        .add_files(file_urls);
                    w
                })
                .await;

            match webhook_res {
                Ok(_) => {
                    println!("Webhook sent successfully, took {:.2?}", start.elapsed());
                }
                Err(e) => {
                    eprintln!("{:?}", e)
                }
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("token");
    let mut client = Client::builder(token)
        .intents(GatewayIntents::GUILD_MESSAGES)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    match client.start().await {
        Ok(_) => print!("Bot woke up!"),
        Err(why) => eprintln!("An error occurred while running the client: {:?}", why),
    }
}
