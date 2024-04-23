use crate::structs::client_structs::{
    Command, Context, Error, EventData, Invocation, InvocationData,
};
use poise;
use poise::serenity_prelude::CreateEmbed;
use poise::serenity_prelude::CreateMessage;

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
