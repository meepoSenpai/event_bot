use std::collections::HashMap;

use poise::serenity_prelude::{self as serenity, prelude::TypeMapKey, UserId};

use super::event::Event;

pub enum Command {
    CreateEvent,
}

pub struct Invocation {
    pub command: Command,
    pub source_channel: serenity::ChannelId,
}

pub struct InvocationData;

impl TypeMapKey for InvocationData {
    type Value = HashMap<UserId, Invocation>;
}

pub struct EventData;

impl TypeMapKey for EventData {
    type Value = Vec<Event>;
}

pub struct Data {}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
