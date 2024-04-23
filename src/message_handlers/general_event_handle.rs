use poise::serenity_prelude::{self as serenity, CacheHttp};

use crate::{
    message_handlers::create_event_handle,
    structs::client_structs::{Command, Data, Error, InvocationData},
};

pub async fn handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    let message_event = match event {
        serenity::FullEvent::Message { new_message } => new_message,
        _ => return Ok(()),
    };
    if let Some(private_channel) = message_event.channel(ctx.http()).await.unwrap().private() {
        let lock_invocation = {
            ctx.data
                .write()
                .await
                .get_mut::<InvocationData>()
                .unwrap()
                .remove(&message_event.author.id)
        };
        if let Some(invocation) = lock_invocation {
            match &invocation.command {
                Command::CreateEvent => {
                    create_event_handle::handle(ctx, message_event, &invocation, &private_channel)
                        .await
                }
            };
        }
    }
    Ok(())
}
