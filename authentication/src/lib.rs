use serde::{Deserialize, Serialize};
use std::{collections::HashMap, os::unix::fs, path::Path};

pub fn greet_user(name: &str) -> String {
    format!("Hello {name}")
}

pub fn hash_password(password: &str) -> String {
    use sha2::Digest;
    let mut hasher = sha2::Sha256::new();
    hasher.update(password);
    return format!("{:X}", hasher.finalize());
}

pub fn get_users() -> HashMap<String, User> {
    return get_users_hashmap();
}

pub fn save_users(users: HashMap<String, User>) {
    let user_path: &Path = Path::new("users.json");
    let users_json: String = serde_json::to_string(&users).unwrap();
    std::fs::write(user_path, users_json).unwrap();
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum LoginRole {
    Admin,
    User,
}

#[derive(PartialEq, Debug, Clone)]
pub enum LoginAction {
    Granted(LoginRole),
    Denied,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub username: String,
    pub password: String,
    pub role: LoginRole,
}

impl User {
    pub fn new(username: &str, password: &str, role: LoginRole) -> User {
        Self {
            username: username.to_lowercase(),
            password: hash_password(password),
            role,
        }
    }
}

fn get_users_array() -> [User; 2] {
    [
        User::new("admin", "password", LoginRole::Admin),
        User::new("bob", "password", LoginRole::User),
    ]
}

fn login_array(username: &str, password: &str) -> Option<LoginAction> {
    let users: [User; 2] = get_users_array();
    let username = username.to_lowercase();

    if let Some(user) = users.iter().find(|user: &&User| user.username == username) {
        if user.password == hash_password(password) {
            return Some(LoginAction::Granted(user.role.clone()));
        }
        return Some(LoginAction::Denied);
    }

    return None;
}

fn get_users_vector() -> Vec<User> {
    vec![
        User::new("admin", "password", LoginRole::Admin),
        User::new("bob", "password", LoginRole::User),
    ]
}

fn login_vector(username: &str, password: &str) -> Option<LoginAction> {
    let users: Vec<User> = get_users_vector();
    let username = username.to_lowercase();

    if let Some(user) = users.iter().find(|user: &&User| user.username == username) {
        if user.password == hash_password(password) {
            return Some(LoginAction::Granted(user.role.clone()));
        }
        return Some(LoginAction::Denied);
    }

    return None;
}

fn get_users_hashmap() -> HashMap<String, User> {
    let users_path: &Path = Path::new("users.json");
    if users_path.exists() {
        let users_json = std::fs::read_to_string(users_path).unwrap();
        return serde_json::from_str(&users_json).unwrap();
    }

    let users = get_default_users();
    let users_json = serde_json::to_string(&users).unwrap();
    std::fs::write(users_path, users_json).unwrap();
    return users;
}

fn get_default_users() -> HashMap<String, User> {
    let mut users: HashMap<String, User> = HashMap::new();

    users.insert(
        "admin".to_string(),
        User::new("admin", "password", LoginRole::Admin),
    );

    users.insert(
        "bob".to_string(),
        User::new("bob", "password", LoginRole::User),
    );

    return users;
}

fn login_hashmap(username: &str, password: &str) -> Option<LoginAction> {
    let users: HashMap<String, User> = get_users_hashmap();
    let username = username.to_lowercase();

    if let Some(user) = users.get(&username) {
        if user.password == hash_password(password) {
            return Some(LoginAction::Granted(user.role.clone()));
        }
        return Some(LoginAction::Denied);
    }

    return None;
}

pub fn login(username: &str, password: &str) -> Option<LoginAction> {
    // return login_array(username, password);
    // return login_vector(username, password);
    return login_hashmap(username, password);
}

pub fn read_line() -> String {
    let mut input: String = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("stdin not working");
    input.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_greet_user() {
        assert_eq!("Hello will", greet_user("will"))
    }

    #[test]
    fn test_login() {
        assert_eq!(
            Some(LoginAction::Granted(LoginRole::Admin)),
            login("AdMiN", "password")
        );
        assert_eq!(
            Some(LoginAction::Granted(LoginRole::User)),
            login("BoB", "password")
        );
        assert_eq!(Some(LoginAction::Denied), login("BoB", "nope"));
        assert_eq!(None, login("user", "password"));
        assert_eq!(Some(LoginAction::Denied), login("admin", "nope"));
    }
}
