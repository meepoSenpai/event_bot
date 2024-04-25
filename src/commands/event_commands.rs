use std::time::Duration;

use crate::structs::client_structs::{
    Command, Context, Error, EventData, Invocation, InvocationData,
};
use crate::structs::event::Event;
use poise;
use poise::serenity_prelude::{ComponentInteractionDataKind, CreateEmbed, MessageFlags};
use poise::serenity_prelude::{
    CreateMessage, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption,
};

/// Create an Event
#[poise::command(slash_command, prefix_command)]
pub async fn create_event(ctx: Context<'_>) -> Result<(), Error> {
    let mut data = ctx.serenity_context().data.write().await;
    let invocation_data = data.get_mut::<InvocationData>().unwrap();
    invocation_data.insert(
        ctx.author().id,
        Invocation {
            command: Command::CreateEvent,
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

/// List all current events for this server
#[poise::command(slash_command, prefix_command)]
pub async fn list_events(ctx: Context<'_>) -> Result<(), Error> {
    let mut data = ctx.serenity_context().data.write().await;
    println!("Got Event Data");
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
    let events_on_server = {
        let event_data = ctx.serenity_context().data.read().await;
        event_data
            .get::<EventData>()
            .unwrap()
            .iter()
            .filter(|ev| ev.server_id() == ctx.guild_id().unwrap())
            .cloned()
            .collect::<Vec<Event>>()
    };
    if events_on_server.is_empty() {
        ctx.say("There are no events to sign up for.").await?;
        return Ok(());
    }
    ctx.say("Select an event to sign up to").await?;
    let message = ctx
        .channel_id()
        .send_message(
            ctx.http(),
            CreateMessage::new()
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
    let possible_roles = events_on_server
        .iter()
        .find(|ev| ev.id.to_string() == event_selection)
        .unwrap()
        .roles();
    let message = ctx
        .channel_id()
        .send_message(
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
    let role_selection = match &interaction.data.kind {
        ComponentInteractionDataKind::StringSelect { values } => String::from(&values[0]),
        _ => panic!("unexpected interaction data kind"),
    };
    message.delete(&ctx).await.unwrap();
    ctx.channel_id()
        .send_message(
            ctx.http(),
            CreateMessage::new()
                .content("Thank you for signing up! We'll handle the rest.")
                .flags(MessageFlags::URGENT),
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
        ctx.say("You are not signed up for any events.").await?;
        return Ok(());
    }
    let message = ctx
        .channel_id()
        .send_message(
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
    ctx.channel_id()
        .send_message(
            ctx.http(),
            CreateMessage::new()
                .content("You have been removed from the event.")
                .flags(MessageFlags::EPHEMERAL),
        )
        .await
        .unwrap();
    Ok(())
}
