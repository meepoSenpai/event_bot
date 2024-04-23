pub mod commands;
pub mod message_handlers;
pub mod structs;
pub mod util;

use crate::message_handlers::general_event_handle::handler;
use commands::event_commands::{create_event, list_events};
use poise::serenity_prelude::{self as serenity};
use poise::{self};
use std::collections::HashMap;
use std::env;
use structs::client_structs::{Data, EventData, InvocationData};

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").unwrap();
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![create_event(), list_events()],
            event_handler: |ctx, event, framework, data| {
                Box::pin(handler(ctx, event, framework, data))
            },
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
        client
            .data
            .write()
            .await
            .insert::<InvocationData>(HashMap::new());
        client.data.write().await.insert::<EventData>(Vec::new());
    }
    client.start().await.unwrap();
}
