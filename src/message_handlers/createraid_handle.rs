use chrono;
use poise::serenity_prelude::futures::StreamExt;

use crate::structs::event::{Event, Role, RoleFlavor};
use crate::util::simple_message_user;
use crate::Invocation;
use poise::serenity_prelude::{
    self as serenity, CacheHttp, ChannelId, Context, CreateMessage, MessageCollector,
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
        match new_message.content.trim().parse::<u32>() {
            Ok(num) => {
                return Some(Role {
                    name: message.content,
                    amount: num,
                })
            }
            Err(_) => {}
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
    );
    match extract_role(private_channel.id, ctx, Some(())).await {
        Some(role) => {
            let to_print = format!("Role {} extracted, need {}.", role.name, role.amount);
            println!("{}", to_print);
            event.add_role(role)
        }
        None => return,
    }
    loop {
        match extract_role(private_channel.id, ctx, None).await {
            Some(role) => event.add_role(role),
            None => break,
        }
    }

    let reply = format!(
        "Event {} was created by {}.\n The following roles are needed:\n{}",
        message.content,
        message.author,
        event
            .needed_roles
            .iter()
            .map(|x| format!("Role: {}, needed: {}", x.name, x.amount))
            .collect::<Vec<String>>()
            .join(", ")
    );
    simple_message_user(ctx, invocation.source_channel, reply).await
}
