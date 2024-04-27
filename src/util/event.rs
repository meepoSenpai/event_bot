use crate::structs::event::Role;
use poise::serenity_prelude::futures::StreamExt;
use poise::serenity_prelude::{CacheHttp, ChannelId, Context, MessageCollector};

pub async fn extract_role(
    channel_id: ChannelId,
    ctx: &Context,
    first_prompt: Option<()>,
) -> Option<Role> {
    let mut message_stream = MessageCollector::new(ctx.shard.clone()).stream();
    match first_prompt {
        Some(_) => {
            channel_id
                .say(ctx.http(), String::from("Name a role for the event."))
                .await
                .unwrap();
        }
        None => {
            channel_id
                .say(
                    ctx.http(),
                    String::from("Do you want to create another role?"),
                )
                .await
                .unwrap();
        }
    };
    message_stream.next().await;
    let message = message_stream.next().await.unwrap();
    if !message.content.trim().to_lowercase().contains("no") {
        let message_stream = &mut message_stream;
        channel_id
            .say(
                ctx.http(),
                format!("How many {} will you need?", message.content),
            )
            .await
            .unwrap();
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
