use super::errors;
use std::collections::HashMap;
use uuid::Uuid;

pub trait TokenSession: Send + Sync {
    fn add_user(&mut self, user: &str) -> Result<Uuid, errors::SessionError>;
    fn get_user(&self, id: &Uuid) -> Option<String>;
}

pub struct DefaultTokenSession {
    active_users: HashMap<Uuid, String>,
}

impl Clone for DefaultTokenSession {
    fn clone(&self) -> Self {
        Self {
            active_users: self.active_users.clone(),
        }
    }
}

impl Default for DefaultTokenSession {
    fn default() -> Self {
        Self {
            active_users: HashMap::new(),
        }
    }
}

impl TokenSession for DefaultTokenSession {
    fn add_user(&mut self, user: &str) -> Result<Uuid, errors::SessionError> {
        let mut uuid = Uuid::new_v4();

        while self.active_users.contains_key(&uuid) {
            uuid = Uuid::new_v4();
        }

        // self.active_users.entry(uuid)
        //     .and_modify(|e| *e = user.to_string());

        self.active_users.insert(uuid, user.to_owned());

        Ok(uuid)
    }
    fn get_user(&self, id: &Uuid) -> Option<String> {
        match self.active_users.get(id) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }
}
