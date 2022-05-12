extern crate dotenv;

use dotenv::dotenv;
use lazy_static::lazy_static;
use serenity::{
    async_trait,
    client::{Client, Context, EventHandler},
    model::{
        channel::{AttachmentType::Image, Message},
        id::ChannelId,
    },
    prelude::GatewayIntents,
};
use std::env;
use tracing::{event, instrument, Level};
use url::Url;


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
    #[instrument(skip(self, ctx, incoming_message), fields(message_id = incoming_message.id.as_u64()))]
    async fn message(&self, ctx: Context, incoming_message: Message) {
        if incoming_message.channel_id == *CHANNEL_ID {
            let webhook = ctx
                .http
                .get_webhook_with_token(*TARGET_CHANNEL_ID, &TARGET_CHANNEL_WEBHOOK_TOKEN)
                .await
                .unwrap();

            event!(Level::INFO, "Webhook generated");

            // Get username or
            let username: String = match incoming_message.author_nick(&ctx.http).await {
                Some(nick) => nick,
                None => incoming_message.author.name.clone(),
            };


            let Message {
                attachments,
                content,
                author,
                activity,
                ..
            } = incoming_message;

            event!(
                Level::INFO,
                "Parsed username {} and content {}",
                username,
                content
            );

            let file_urls =
                attachments
                    .iter()
                    .filter_map(|attachment| match Url::parse(&attachment.url) {
                        Ok(url) => Some(Image(url)),
                        Err(err) => {
                            event!(
                                Level::WARN,
                                "Failed to parse the attachment url {}, error {}",
                                &attachment.url, err
                            );
                            None
                        }
                    });

            event!(Level::INFO, "Parsed file urls",);

            let webhook_res = webhook
                .execute(&ctx.http, false, |w| {
                    event!(Level::INFO, "Sending message to webhook, content: {content}, username: {username}");
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
                    event!(Level::INFO, "Webhook sent successfully");
                }
                Err(e) => {
                    event!(Level::ERROR, "{:?}", e)
                }
            }
        } else {
            event!(Level::INFO, "No message sent");
        }
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    dotenv().ok();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();

    let token = env::var("DISCORD_TOKEN").expect("token");
    let mut client = Client::builder(
        token,
        GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT,
    )
    .event_handler(Handler)
    .await
    .expect("Error creating client");

    // start listening for events by starting a single shard
    match client.start().await {
        Ok(_) => event!(Level::INFO, "Bot woke up!"),
        Err(why) => event!(
            Level::ERROR,
            "An error occurred while running the client: {:?}",
            why
        ),
    }
}
