use dashmap::DashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct User {
    pub username: String,
}

pub type UserStore = Arc<DashMap<Uuid, User>>;
pub type UsernameStore = Arc<DashMap<String, Uuid>>;

pub fn create_user_stores() -> (UserStore, UsernameStore) {
    (Arc::new(DashMap::new()), Arc::new(DashMap::new()))
}

pub fn add_user(
    users: &UserStore,
    usernames: &UsernameStore,
    username: String,
) -> Result<Uuid, String> {
    if usernames.contains_key(&username) {
        return Err(format!("Username '{username}' is already taken"));
    }

    let user_id = Uuid::new_v4();
    let user = User {
        username: username.clone(),
    };

    users.insert(user_id, user);
    usernames.insert(username, user_id);

    Ok(user_id)
}

pub fn remove_user(users: &UserStore, usernames: &UsernameStore, user_id: Uuid) -> Option<String> {
    if let Some((_, user)) = users.remove(&user_id) {
        usernames.remove(&user.username);
        Some(user.username)
    } else {
        None
    }
}

pub fn get_user(users: &UserStore, user_id: Uuid) -> Option<User> {
    users.get(&user_id).map(|u| u.value().clone())
}
