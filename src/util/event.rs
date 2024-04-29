use std::time::Duration;

use crate::structs::client_structs::Context as pContext;
use crate::structs::event::Role;
use chrono::{DateTime, NaiveTime, Timelike};
use poise::serenity_prelude::{futures::StreamExt, ComponentInteractionDataKind};
use poise::serenity_prelude::{
    CacheHttp, ChannelId, Context, CreateMessage, CreateSelectMenu, CreateSelectMenuKind,
    CreateSelectMenuOption, MessageCollector,
};

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

pub async fn extract_datetime(channel_id: ChannelId, ctx: &pContext<'_>) -> DateTime<chrono::Utc> {
    let handler = channel_id
        .send_message(
            ctx.http(),
            CreateMessage::new()
                .content("When is the event?")
                .select_menu(CreateSelectMenu::new(
                    "EventDate",
                    CreateSelectMenuKind::String {
                        options: [1,2,3,4,5,6,7]
                            .iter()
                            .map(|num| {
                                let day = chrono::Local::now() + chrono::Duration::days(*num);
                                CreateSelectMenuOption::new(
                                    day.format("%A %B %d, %Y").to_string(),
                                    day.format("%Y-%m-%d %H:%M:%S %z").to_string(),
                                )
                            })
                            .collect(),
                    },
                )),
        )
        .await
        .unwrap();
    let interaction = handler
        .await_component_interaction(ctx.serenity_context().shard.clone())
        .timeout(Duration::from_secs(30))
        .await
        .unwrap();
    let x = match &interaction.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => String::from(&values[0]),
        _ => panic!("unexpected interaction data kind"),
    };
    handler.delete(&ctx).await.unwrap();
    let selected_date = DateTime::parse_from_str(&x, "%Y-%m-%d %H:%M:%S %z")
        .unwrap()
        .to_utc();
    let handler = channel_id
        .send_message(
            ctx.http(),
            CreateMessage::new()
                .content("At what time is the event?")
                .select_menu(CreateSelectMenu::new(
                    "EventTime",
                    CreateSelectMenuKind::String {
                        options: (0..24)
                            .map(|num| {
                                let day = chrono::Local::now().with_time(NaiveTime::default().with_hour(0).unwrap().with_minute(0).unwrap().with_second(0).unwrap().with_nanosecond(0).unwrap()).unwrap() + chrono::Duration::hours(num);
                                println!("{}", day.format("%Y-%m-%d %H:%M:%S %z"));
                                CreateSelectMenuOption::new(
                                    day.format("%H:%M").to_string(),
                                    day.format("%Y-%m-%d %H:%M:%S %z").to_string(),
                                )
                            })
                            .collect(),
                    },
                )),
        )
        .await
        .unwrap();
    let x = match &interaction.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => String::from(&values[0]),
        _ => panic!("unexpected interaction data kind"),
    };
    handler.delete(&ctx).await.unwrap();
    let selected_time = DateTime::parse_from_str(&x, "%Y-%m-%d %H:%M:%S %z")
        .unwrap()
        .to_utc();
    return selected_date.with_time(selected_time.time()).unwrap();
}
