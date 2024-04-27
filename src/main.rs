pub mod commands;
pub mod structs;
pub mod util;

use commands::event_commands::{create_event, list_events, sign_off, sign_up};
use poise::serenity_prelude::{self as serenity};
use poise::{self};
use std::env;
use structs::client_structs::{Data, EventData};

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").unwrap();
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![create_event(), list_events(), sign_up(), sign_off()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let mut client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await
        .unwrap();
    {
        let client = &mut client;
        client.data.write().await.insert::<EventData>(Vec::new());
    }
    client.start().await.unwrap();
}
