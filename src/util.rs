use poise::serenity_prelude::{CacheHttp, ChannelId, Context, CreateMessage};

pub async fn simple_message_user(ctx: &Context, channel_id: ChannelId, prompt: String) {
    channel_id
        .send_message(ctx.http(), CreateMessage::new().content(prompt))
        .await
        .unwrap();
}
