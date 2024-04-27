use std::time::Duration;

use crate::structs::client_structs::{Context, Error, EventData};
use crate::structs::event::Event;
use crate::util::event::extract_role;
use poise;
use poise::serenity_prelude::{ComponentInteractionDataKind, CreateChannel, CreateEmbed, MessageFlags};
use poise::serenity_prelude::{
    CreateMessage, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
};

/// Create an Event
#[poise::command(slash_command, prefix_command)]
pub async fn create_event(ctx: Context<'_>, create_new_channel: Option<bool>) -> Result<(), Error> {
    ctx.say("Thinking...").await?.delete(ctx).await?;
    let private_cannel = ctx.author().create_dm_channel(ctx.http()).await?;
    private_cannel
        .say(ctx.http(), "How would you like to name the Event?")
        .await?;
    let event_name = {
        let message_stream = private_cannel
            .id
            .await_reply(ctx.serenity_context().shard.clone());
        message_stream.next().await.unwrap().content
    };
    let mut event = Event::new(
        ctx.author().clone(),
        event_name,
        chrono::Local::now(),
        ctx.guild_id().unwrap(),
    );
    match extract_role(private_cannel.id, ctx.serenity_context(), Some(())).await {
        Some(role) => event.add_role(role),
        None => {
            ctx.author()
                .dm(
                    ctx.http(),
                    CreateMessage::new().content("Could not create event."),
                )
                .await?;
            return Ok(());
        }
    }
    while let Some(role) = extract_role(private_cannel.id, ctx.serenity_context(), None).await {
        event.add_role(role);
    }
    let channel = match create_new_channel {
        Some(_) => ctx.guild_id().unwrap().create_channel(ctx.http(), CreateChannel::new(&event.title)).await.unwrap(),
        None => ctx.clone().guild_channel().await.unwrap()
    };
    let reply = format!("\nThe following event was created by {}:", ctx.author());
    ctx.say(reply).await?;
    let event_message = CreateMessage::new().add_embed(
        CreateEmbed::new()
            .title(&event.title)
            .description(event.build_new_message()),
    );
    event.add_event_message(
        channel.id
            .send_message(ctx.http(), event_message)
            .await
            .unwrap(),
    );
    if let Some(event_data) = ctx
        .serenity_context()
        .data
        .write()
        .await
        .get_mut::<EventData>()
    {
        event_data.push(event)
    }
    Ok(())
}

/// List all current events for this server
#[poise::command(slash_command, prefix_command)]
pub async fn list_events(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Thinking...").await?.delete(ctx).await?;
    let mut data = ctx.serenity_context().data.write().await;
    for event in data.get_mut::<EventData>().unwrap() {
        if event.server_id() != ctx.guild_id().unwrap() {
            continue;
        }
        let event_message = ctx
            .channel_id()
            .send_message(
                ctx.http(),
                CreateMessage::new().add_embed(
                    CreateEmbed::new()
                        .title(&event.title)
                        .description(event.build_new_message()),
                ),
            )
            .await
            .unwrap();
        event.add_event_message(event_message);
    }
    Ok(())
}

/// Signup for an event
#[poise::command(slash_command, prefix_command)]
pub async fn sign_up(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Thinking...").await?.delete(ctx).await?;
    let events_on_server = {
        let event_data = ctx.serenity_context().data.read().await;
        event_data
            .get::<EventData>()
            .unwrap()
            .iter()
            .filter(|ev| ev.server_id() == ctx.guild_id().unwrap())
            .filter(|ev| !ev.contains_participant(ctx.author()))
            .cloned()
            .collect::<Vec<Event>>()
    };
    if events_on_server.is_empty() {
        ctx.author()
            .dm(
                ctx.http(),
                CreateMessage::new().content("There are no events to sign up for."),
            )
            .await?;
        return Ok(());
    }
    let message = ctx
        .author()
        .dm(
            ctx.http(),
            CreateMessage::new()
                .content("Select an event to sign up for.")
                .select_menu(CreateSelectMenu::new(
                    "event_select",
                    CreateSelectMenuKind::String {
                        options: events_on_server
                            .iter()
                            .map(|ev| {
                                CreateSelectMenuOption::new(ev.title.clone(), ev.id.to_string())
                            })
                            .collect(),
                    },
                )),
        )
        .await
        .unwrap();
    let interaction = message
        .await_component_interaction(&ctx.serenity_context().shard)
        .timeout(Duration::from_secs(30))
        .await
        .unwrap();
    let event_selection = match &interaction.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => String::from(&values[0]),
        _ => panic!("unexpected interaction data kind"),
    };
    message.delete(&ctx).await.unwrap();
    let possible_roles = events_on_server
        .iter()
        .find(|ev| ev.id.to_string() == event_selection)
        .unwrap()
        .roles();
    let message = ctx
        .author()
        .dm(
            ctx.http(),
            CreateMessage::new()
                .content("Please Pick a Role for the event")
                .select_menu(CreateSelectMenu::new(
                    "role_select",
                    CreateSelectMenuKind::String {
                        options: possible_roles
                            .iter()
                            .map(|rl| CreateSelectMenuOption::new(rl, rl))
                            .collect(),
                    },
                )),
        )
        .await
        .unwrap();
    let interaction = message
        .await_component_interaction(&ctx.serenity_context().shard)
        .timeout(Duration::from_secs(30))
        .await
        .unwrap();
    let role_selection = match &interaction.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => String::from(&values[0]),
        _ => panic!("unexpected interaction data kind"),
    };
    message.delete(&ctx).await.unwrap();
    ctx.author()
        .dm(
            ctx.http(),
            CreateMessage::new().content("Thank you for signing up! We'll handle the rest."),
        )
        .await
        .unwrap();
    if let Some(event_data) = ctx
        .serenity_context()
        .data
        .write()
        .await
        .get_mut::<EventData>()
    {
        if let Some(event) = event_data
            .iter_mut()
            .find(|ev| ev.id.to_string() == event_selection)
        {
            event
                .add_participant(ctx.author().clone(), role_selection, String::from(""))
                .unwrap();
            event.update_event_messages(ctx.http()).await;
        }
    }
    Ok(())
}

/// Sign off from an event
#[poise::command(slash_command, prefix_command)]
pub async fn sign_off(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Thinking...").await?.delete(ctx).await?;
    let events_on_server = {
        let event_data = ctx.serenity_context().data.read().await;
        event_data
            .get::<EventData>()
            .unwrap()
            .iter()
            .filter(|ev| ev.server_id() == ctx.guild_id().unwrap())
            .filter(|ev| ev.contains_participant(ctx.author()))
            .cloned()
            .collect::<Vec<Event>>()
    };
    if events_on_server.is_empty() {
        ctx.author()
            .dm(
                ctx,
                CreateMessage::new().content("You are not signed up for any events."),
            )
            .await?;
        return Ok(());
    }
    let message = ctx
        .author()
        .dm(
            ctx.http(),
            CreateMessage::new()
                .content("\nPlease Pick an Event to sign off from")
                .select_menu(CreateSelectMenu::new(
                    "event_select",
                    CreateSelectMenuKind::String {
                        options: events_on_server
                            .iter()
                            .cloned()
                            .map(|ev| CreateSelectMenuOption::new(ev.title, ev.id.to_string()))
                            .collect(),
                    },
                ))
                .flags(MessageFlags::EPHEMERAL),
        )
        .await
        .unwrap();
    let interaction = message
        .await_component_interaction(&ctx.serenity_context().shard)
        .timeout(Duration::from_secs(30))
        .await
        .unwrap();
    let event_selection = match &interaction.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => String::from(&values[0]),
        _ => panic!("unexpected interaction data kind"),
    };
    message.delete(&ctx).await.unwrap();
    if let Some(event_data) = ctx
        .serenity_context()
        .data
        .write()
        .await
        .get_mut::<EventData>()
    {
        if let Some(event) = event_data
            .iter_mut()
            .find(|ev| ev.id.to_string() == event_selection)
        {
            event.remove_participant(ctx.author().clone()).unwrap();
            event.update_event_messages(ctx.http()).await;
        }
    }
    ctx.author()
        .dm(
            ctx.http(),
            CreateMessage::new()
                .content("You have been removed from the event.")
                .flags(MessageFlags::EPHEMERAL),
        )
        .await
        .unwrap();
    Ok(())
}
