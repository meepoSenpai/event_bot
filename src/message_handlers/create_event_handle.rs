use chrono;
use poise::serenity_prelude::futures::StreamExt;

use crate::structs::client_structs::{EventData, Invocation};
use crate::structs::event::{Event, Role};
use crate::util::simple_message_user;
use poise::serenity_prelude::{
    CacheHttp, ChannelId, Context, CreateEmbed, CreateMessage, MessageCollector,
};
use poise::serenity_prelude::{Message, PrivateChannel};

async fn extract_role(
    channel_id: ChannelId,
    ctx: &Context,
    first_prompt: Option<()>,
) -> Option<Role> {
    let mut message_stream = MessageCollector::new(ctx.shard.clone()).stream();
    match first_prompt {
        Some(_) => {
            simple_message_user(ctx, channel_id, String::from("Name a role for the event.")).await;
        }
        None => {
            simple_message_user(
                ctx,
                channel_id,
                String::from("Do you want to create another role?"),
            )
            .await;
        }
    };
    message_stream.next().await;
    let message = message_stream.next().await.unwrap();
    if !message.content.trim().to_lowercase().contains("no") {
        let message_stream = &mut message_stream;
        simple_message_user(
            ctx,
            channel_id,
            format!("How many {} will you need?", message.content),
        )
        .await;
        message_stream.next().await;
        let new_message = message_stream.next().await.unwrap();
        if let Ok(num) = new_message.content.trim().parse::<u32>() {
            return Some(Role {
                name: message.content,
                amount: num,
            });
        }
    }
    None
}

pub async fn handle(
    ctx: &Context,
    message: &Message,
    invocation: &Invocation,
    private_channel: &PrivateChannel,
) {
    let mut event = Event::new(
        message.author.clone(),
        message.content.clone(),
        chrono::Local::now(),
        ctx.http()
            .get_channel(invocation.source_channel)
            .await
            .unwrap()
            .guild()
            .unwrap()
            .guild_id,
    );
    match extract_role(private_channel.id, ctx, Some(())).await {
        Some(role) => event.add_role(role),
        None => return,
    }
    while let Some(role) = extract_role(private_channel.id, ctx, None).await {
        event.add_role(role);
    }

    let reply = format!("\nThe following event was created by {}:", message.author);
    simple_message_user(ctx, invocation.source_channel, reply).await;
    let event_message = CreateMessage::new().add_embed(
        CreateEmbed::new()
            .title(&event.title)
            .description(event.build_new_message()),
    );
    event.add_event_message(
        invocation
            .source_channel
            .send_message(ctx.http(), event_message)
            .await
            .unwrap(),
    );
    if let Some(event_data) = ctx.data.write().await.get_mut::<EventData>() {
        event_data.push(event)
    }
}
