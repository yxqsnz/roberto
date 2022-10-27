mod conversation;

use std::env;
use std::sync::Arc;

use conversation::MLChat;
use serenity::async_trait;
use serenity::framework::StandardFramework;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    // Set a handler for the `message` event - so that whenever a new message
    // is received - the closure (or function) passed will be called.
    //
    // Event handlers are dispatched through a threadpool, and so multiple
    // events can be dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        if !msg.content.starts_with(">>") && !msg.author.bot && !msg.content.is_empty() {
            if let Err(e) = msg.channel_id.start_typing(&ctx.http) {
                eprintln!(">>=[error]: {e:?}")
            }

            let data = ctx.data.read().await;
            let chat = data.get::<MLChat>().unwrap();
            let result = msg
                .reply_ping(
                    &ctx.http,
                    chat.ask(msg.content.clone())
                        .await
                        .unwrap_or(String::from("<no response>")),
                )
                .await;
            if let Err(e) = result {
                eprintln!(">>=[error]: {e:?}")
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a
    // shard is booted, and a READY payload is sent by Discord. This payload
    // contains data like the current user's guild Ids, current user data,
    // private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let framework = StandardFramework::new().configure(|c| c.prefix("~"));

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<MLChat>(Arc::new(MLChat::new()));
    }
    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
