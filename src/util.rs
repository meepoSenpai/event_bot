use poise::serenity_prelude::{
    CacheHttp, ChannelId, Context, CreateMessage, Message, MessageCollector, PrivateChannel,
    ShardMessenger,
};

pub async fn simple_message_user(ctx: &Context, channel_id: ChannelId, prompt: String) {
    ctx.http()
        .send_message(channel_id, vec![], &CreateMessage::new().content(prompt))
        .await
        .unwrap();
}
