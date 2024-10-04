use super::errors;
use actix_web::guard;
use chrono::DateTime;
use chrono::Local;
use chrono::Timelike;
use chrono::Utc;
use log::debug;
use log::log;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::io::BufRead;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread::JoinHandle;
use std::time::Duration;
use tokio::sync::oneshot::{Receiver, Sender};
use uuid::Uuid;
type SessionMap = HashMap<Uuid, UserSessionData>;

#[derive(Clone, Debug)]
struct UserSessionData {
    user_login: String,
    expire: DateTime<Utc>,
}

pub trait TokenSession: Send + Sync {
    fn add_user(&mut self, user: &str) -> Uuid;
    fn get_user(&mut self, id: &Uuid) -> Option<String>;
    fn remove_user(&mut self, user: &Uuid) -> ();
}

pub struct DefaultTokenSession {
    active_users: Arc<Mutex<SessionMap>>,
    cleaner: Option<tokio::task::JoinHandle<()>>,
}

async fn cleaner_task(users_sess: std::sync::Weak<Mutex<SessionMap>>, interval: u64) {
    loop {
        tokio::time::sleep(Duration::from_secs(interval)).await;
        let Some(users) = users_sess.upgrade() else {
            break;
        };
        let mut lock = users.lock();
        let guard = lock.as_mut().unwrap();
        let now = chrono::Utc::now();
        let to_rm = guard
            .iter()
            .filter_map(|(id, usd)| {
                if now.ge(&usd.expire) {
                    Some(id.to_owned())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        for id in to_rm.into_iter() {
            guard.remove(&id);
        }
        drop(lock);
    }
}

impl DefaultTokenSession {
    ///
    /// `cleaning_interval` determines if a cleaner task should be spawned
    /// and how big the time interval between each cleaning is.
    ///
    pub fn new(cleaning_interval: Option<u64>) -> Self {
        let users = Arc::new(Mutex::new(SessionMap::new()));
        let mut clean: Option<tokio::task::JoinHandle<()>> = None;
        if let Some(interval) = cleaning_interval {
            let users_for_cleanup = users.clone();
            clean = Some(tokio::spawn(async move {
                let weak = Arc::downgrade(&users_for_cleanup);
                cleaner_task(weak, interval).await
            }));
        }
        Self {
            active_users: users.clone(),
            cleaner: clean,
        }
    }
}

impl TokenSession for DefaultTokenSession {
    fn add_user(&mut self, user: &str) -> Uuid {
        let mut uuid = Uuid::new_v4();

        let mut lock = self.active_users.lock();
        let active_users = lock.as_mut().expect("mutex poisoned");

        while active_users.contains_key(&uuid) {
            uuid = Uuid::new_v4();
        }

        let session_data = UserSessionData {
            user_login: user.to_owned(),
            expire: chrono::Utc::now() + Duration::from_secs(600),
        };

        active_users.insert(uuid, session_data);

        uuid
    }
    fn get_user(&mut self, id: &Uuid) -> Option<String> {
        let mut lock = self.active_users.lock();
        let active_users = lock.as_mut().expect("mutext poisoned");
        if active_users.contains_key(id) {
            active_users.entry(*id).and_modify(|e| {
                e.expire += Duration::from_secs(60 * 5);
            });
            Some(active_users.get(id).unwrap().user_login.to_owned())
        } else {
            None
        }
    }
    fn remove_user(&mut self, user: &Uuid) -> () {
        let mut lock = self.active_users.lock();
        let active_users = lock.as_mut().expect("mutext poisoned");
        active_users.remove(user);
        debug!("has_user {}", active_users.contains_key(user));
    }
}
