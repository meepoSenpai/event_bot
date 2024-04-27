use poise::serenity_prelude::prelude::TypeMapKey;

use super::event::Event;

pub enum Command {
    CreateEvent,
}

pub struct EventData;

impl TypeMapKey for EventData {
    type Value = Vec<Event>;
}

pub struct Data {}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
