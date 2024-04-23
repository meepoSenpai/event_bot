use chrono;
use chrono::DateTime;
use poise::serenity_prelude::{
    self as serenity, CacheHttp, CreateEmbed, EditMessage, GuildId, Message,
};
use serenity::model::user::User;
use uuid::Uuid;

pub struct Event {
    pub creator: User,
    pub title: String,
    pub id: Uuid,
    server_id: GuildId,
    pub date: DateTime<chrono::Local>,
    pub participants: Vec<Participant>,
    pub needed_roles: Vec<Role>,
    pub needed_flavors: Vec<RoleFlavor>,
    event_messages: Vec<Message>,
}

impl Event {
    pub fn new(
        creator: User,
        title: String,
        date: DateTime<chrono::Local>,
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
        let x = self.needed_roles.iter().map(|x| {
            format!(
                "{}: {}/{}",
                x.name,
                self.participants
                    .iter()
                    .filter(|y| y.role.name == x.name)
                    .count(),
                x.amount
            )
        });
        for elem in x {
            role_strings.push(elem);
        }

        format!(
            "\n
        ID: {}\n
        Needed Roles:\n
        {}
        ",
            self.id,
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
        self.participants.push(Participant {
            id: user,
            role: user_role,
            flavor: user_flavor,
        });
        Ok(())
    }
}

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
