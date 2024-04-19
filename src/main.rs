use std::collections::HashMap;
use std::env;

use poise::serenity_prelude::prelude::TypeMapKey;
use poise::serenity_prelude::{self as serenity};
use poise::serenity_prelude::{CacheHttp, UserId};
use poise::{self};

pub mod message_handlers;
pub mod structs;
pub mod util;

use message_handlers::createraid_handle;
use structs::event::Event;

enum Command {
    CreateRaid,
}

pub struct Invocation {
    command: Command,
    source_channel: serenity::ChannelId,
}

struct InvocationData;

impl TypeMapKey for InvocationData {
    type Value = HashMap<UserId, Invocation>;
}

struct EventData;

impl TypeMapKey for EventData {
    type Value = Vec<Event>;
}

struct Data {}
// User data, which is stored and accessible in all command invocations

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Create a Raid Event
#[poise::command(slash_command, prefix_command)]
async fn createraid(ctx: Context<'_>) -> Result<(), Error> {
    let mut data = ctx.serenity_context().data.write().await;
    let invocation_data = data.get_mut::<InvocationData>().unwrap();
    invocation_data.insert(
        ctx.author().id,
        Invocation {
            command: Command::CreateRaid,
            source_channel: ctx.channel_id(),
        },
    );
    let private_cannel = ctx.author().create_dm_channel(ctx.http()).await?;
    private_cannel
        .say(ctx.http(), "How would you like to name the Event?")
        .await?;
    ctx.say("Helping you in a bit my friend!").await?;
    Ok(())
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    let message_event = match event {
        serenity::FullEvent::Message { new_message } => new_message,
        _ => return Ok(()),
    };
    match message_event.channel(ctx.http()).await.unwrap().private() {
        Some(private_channel) => {
            match ctx
                .data
                .write()
                .await
                .get_mut::<InvocationData>()
                .unwrap()
                .remove(&message_event.author.id)
            {
                Some(invocation) => {
                    match &invocation.command {
                        Command::CreateRaid => {
                            createraid_handle::handle(
                                ctx,
                                message_event,
                                &invocation,
                                &private_channel,
                            )
                            .await
                        }
                    };
                }
                _ => {}
            }
        }
        _ => return Ok(()),
    };
    return Ok(());
}

#[tokio::main]
async fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![createraid()],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
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
    client
        .data
        .write()
        .await
        .insert::<InvocationData>(HashMap::new());
    client.data.write().await.insert::<EventData>(Vec::new());
    client.start().await.unwrap();
}
