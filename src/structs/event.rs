use chrono::DateTime;
use chrono::{self};
use poise::serenity_prelude::{
    self as serenity, CacheHttp, CreateEmbed, EditMessage, GuildId, Message,
};
use serenity::model::user::User;
use uuid::Uuid;

#[derive(Clone)]
pub struct Event {
    pub creator: User,
    pub title: String,
    pub id: Uuid,
    server_id: GuildId,
    date: DateTime<chrono::Utc>,
    participants: Vec<Participant>,
    needed_roles: Vec<Role>,
    needed_flavors: Vec<RoleFlavor>,
    event_messages: Vec<Message>,
}

impl Event {
    pub fn new(
        creator: User,
        title: String,
        date: DateTime<chrono::Utc>,
        server_id: GuildId,
    ) -> Event {
        Event {
            creator,
            title,
            date,
            server_id,
            id: Uuid::new_v4(),
            participants: Vec::new(),
            needed_roles: Vec::new(),
            needed_flavors: Vec::new(),
            event_messages: Vec::new(),
        }
    }

    pub fn needed_participants(&self) -> u32 {
        self.needed_roles.iter().map(|x| x.amount).sum::<u32>()
    }

    pub fn server_id(&self) -> GuildId {
        self.server_id
    }

    pub fn add_role(&mut self, role: Role) {
        self.needed_roles.push(role);
    }

    pub fn add_flavor(&mut self, flavor: RoleFlavor) {
        self.needed_flavors.push(flavor);
    }

    pub fn add_event_message(&mut self, new_message: Message) {
        self.event_messages.push(new_message);
    }

    pub async fn update_event_messages(&mut self, http: &impl CacheHttp) {
        let message = EditMessage::new().embed(
            CreateEmbed::new()
                .title(&self.title)
                .description(self.build_new_message()),
        );
        for l in self
            .event_messages
            .iter_mut()
            .map(|x| x.edit(http, message.clone()))
        {
            l.await.unwrap();
        }
    }

    pub fn build_new_message(&self) -> String {
        let mut role_strings = Vec::<String>::new();
        for role in self.needed_roles.iter() {
            let participant_iter = self
                .participants
                .iter()
                .filter(|prt| prt.role.name == role.name);
            role_strings.push(format!(
                "{}: {}/{}",
                role.name,
                participant_iter.clone().count(),
                role.amount
            ));
            for participant in participant_iter {
                role_strings.push(format!(
                    "→ {}{}",
                    &participant
                        .flavor
                        .clone()
                        .unwrap_or(RoleFlavor {
                            flavor: "".to_string(),
                            amount: 0
                        })
                        .flavor,
                    participant.id
                ));
            }
        }

        format!(
            "\n
        Date: <t:{}>
        ({} / {})
        Needed Roles:\n
        {}
        ",
            self.date.timestamp(),
            self.participants.len(),
            self.needed_roles
                .iter()
                .map(|role| role.amount)
                .sum::<u32>(),
            role_strings.join("\n")
        )
    }

    pub fn add_participant(
        &mut self,
        user: User,
        role: String,
        flavor: String,
    ) -> Result<(), &str> {
        let user_role = match self.needed_roles.iter().find(|rl| rl.name == role) {
            Some(rl) => rl.clone(),
            None => return Err("No role with that name found."),
        };
        let user_flavor = match self.needed_flavors.iter().find(|flv| flv.flavor == flavor) {
            Some(flv) => Some(flv.clone()),
            None => {
                if !flavor.is_empty() {
                    return Err("No flavor with that name found.");
                }
                None
            }
        };
        if self.is_full() {
            return Err("Event is full.");
        }
        if self.is_role_full(&user_role.name) {
            return Err("Role is full.");
        }
        if let Some(flv) = &user_flavor {
            if self.is_flavor_full(&flv.flavor) {
                return Err("Flavor is full.");
            }
        }
        self.participants.push(Participant {
            id: user,
            role: user_role,
            flavor: user_flavor,
        });
        Ok(())
    }

    fn is_full(&self) -> bool {
        self.participants.len() == self.needed_roles.iter().map(|x| x.amount).sum::<u32>() as usize
    }

    pub fn is_role_full(&self, role: &String) -> bool {
        self.participants
            .iter()
            .filter(|x| x.role.name == *role)
            .count() as u32
            >= self
                .needed_roles
                .iter()
                .find(|x| x.name == *role)
                .unwrap()
                .amount
    }

    pub fn is_flavor_full(&self, flavor: &String) -> bool {
        let flavor = self
            .needed_flavors
            .iter()
            .find(|x| x.flavor == *flavor)
            .cloned()
            .unwrap_or(RoleFlavor {
                flavor: "".to_string(),
                amount: 0,
            });
        flavor.amount == 0
            || self
                .participants
                .iter()
                .filter(|x| {
                    x.flavor
                        .clone()
                        .unwrap_or(RoleFlavor {
                            flavor: "".to_string(),
                            amount: 0,
                        })
                        .flavor
                        == flavor.flavor
                })
                .count() as u32
                >= flavor.amount
    }

    pub fn remove_participant(&mut self, user: User) -> Result<(), &str> {
        let index = self
            .participants
            .iter()
            .position(|x| x.id.id == user.id)
            .ok_or("User not found.")?;
        self.participants.remove(index);
        Ok(())
    }

    pub fn contains_participant(&self, user: &User) -> bool {
        self.participants.iter().any(|x| x.id.id == user.id)
    }

    pub fn roles(&self) -> Vec<String> {
        self.needed_roles.iter().map(|x| x.name.clone()).collect()
    }

    pub fn flavors(&self) -> Vec<String> {
        self.needed_flavors
            .iter()
            .map(|x| x.flavor.clone())
            .collect()
    }

    pub fn possible_roles(&self) -> Vec<String> {
        self.needed_roles
            .iter()
            .filter(|x| !self.is_role_full(&x.name))
            .map(|x| x.name.clone())
            .collect()
    }

    pub fn possible_flavors(&self) -> Vec<String> {
        let mut possible_flavors: Vec<String> = self
            .needed_flavors
            .iter()
            .filter(|x| !self.is_flavor_full(&x.flavor))
            .cloned()
            .map(|x| x.flavor.clone())
            .collect();
        possible_flavors.push("None".to_string());
        possible_flavors
    }
}

#[derive(Clone)]
pub struct Participant {
    pub id: User,
    pub role: Role,
    pub flavor: Option<RoleFlavor>,
}

#[derive(Clone)]
pub struct Role {
    pub name: String,
    pub amount: u32,
}

#[derive(Clone)]
pub struct RoleFlavor {
    pub flavor: String,
    pub amount: u32,
}
