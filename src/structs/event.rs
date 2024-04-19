use chrono;
use chrono::{DateTime, Datelike};
use poise::serenity_prelude as serenity;
use serenity::model::user::User;
use std::collections::HashMap;

pub struct Event {
    pub creator: User,
    pub title: String,
    pub date: DateTime<chrono::Local>,
    pub participants: Vec<Participant>,
    pub needed_roles: Vec<Role>,
    pub needed_flavors: Vec<RoleFlavor>,
}

impl Event {
    pub fn new(creator: User, title: String, date: DateTime<chrono::Local>) -> Event {
        Event {
            creator: creator,
            title: title,
            date: date,
            participants: Vec::new(),
            needed_roles: Vec::new(),
            needed_flavors: Vec::new(),
        }
    }

    pub fn needed_participants(&self) -> u32 {
        self.needed_roles.iter().map(|x| x.amount).sum::<u32>()
    }

    pub fn add_role(&mut self, role: Role) {
        self.needed_roles.push(role);
    }
}

pub struct Participant {
    pub id: User,
    pub role: Role,
    pub flavor: RoleFlavor,
}

pub struct Role {
    pub name: String,
    pub amount: u32,
}

pub struct RoleFlavor {
    pub flavor: String,
    pub amount: u32,
}
