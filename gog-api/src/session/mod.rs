
use super::errors;
use std::borrow::Borrow;
use std::collections::HashMap;
use uuid::Uuid;
use std::sync::Arc;
use std::sync::Mutex;
pub trait TokenSession: Send + Sync {
    fn add_user(&mut self, user: &str) -> Uuid;
    fn get_user(&self, id: &Uuid) -> Option<String>;
    fn remove_user(&mut self, user: &Uuid) -> ();
}

pub struct DefaultTokenSession {
    active_users: Arc<Mutex<HashMap<Uuid, String>>>,
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
        let users = Arc::new(Mutex::new(HashMap::new()));
        let clean = async move {
            
        };
        Self {
            active_users: users.clone(),
        }
    }
}

impl TokenSession for DefaultTokenSession {
    fn add_user(&mut self, user: &str) -> Uuid {
        let mut uuid = Uuid::new_v4();

        let mut lock = self.active_users.lock();
        let active_users = lock.as_mut()
            .expect("mutex poisoned");

        while active_users.contains_key(&uuid) {
            uuid = Uuid::new_v4();
        }

        active_users.insert(uuid, user.to_owned());

        uuid
    }
    fn get_user(&self, id: &Uuid) -> Option<String> {
        let lock = self.active_users.lock();
        let active_users = lock.as_ref().expect("mutext poisoned");
        match active_users.get(id) {
            Some(v) => Some(v.clone()),
            None => None,
        }
    }
    fn remove_user(&mut self, user: &Uuid) -> () {
        let mut lock = self.active_users.lock();
        let active_users = lock.as_mut().expect("mutext poisoned");
        active_users.remove(user);
    }
}
