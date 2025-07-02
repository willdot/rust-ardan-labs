use std::collections::HashMap;

use authentication::{LoginRole, User, get_users, hash_password, save_users};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command()]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Lists all users
    List,
    /// Add a user
    Add {
        /// The users login name
        username: String,
        /// The users password (plain text)
        password: String,
        /// Optional - Mark as an admin
        #[arg{long}]
        admin: Option<bool>,
    },
    /// Delete a user
    Delete {
        /// The username of the user to delete
        username: String,
    },
    /// Change a users password
    ChangePassword {
        /// The username of the user to change
        username: String,
        /// The new password
        new_password: String,
    },
}

fn list_users() {
    println!("{:<20}{:<20}", "Username", "Role");
    println!("{:-<40}", "");

    let users = get_users();

    users
        .iter()
        .for_each(|(_, user)| println!("{:<20}{:<20?}", user.username, user.role));
}

fn add_user(username: String, password: String, admin: bool) {
    let mut users: HashMap<String, User> = get_users();
    let role: LoginRole = if admin {
        LoginRole::Admin
    } else {
        LoginRole::User
    };
    let user = User::new(&username, &password, role);
    users.insert(username, user);
    save_users(users);
}

fn delete_user(username: String) {
    let mut users: HashMap<String, User> = get_users();
    if !users.contains_key(&username) {
        println!("{username} does not exist");
        return;
    }
    users.remove(&username).unwrap();
    save_users(users);
}

fn change_password(username: String, new_password: String) {
    let mut users: HashMap<String, User> = get_users();
    if let Some(user) = users.get_mut(&username) {
        user.password = authentication::hash_password(&new_password);
        save_users(users);
    } else {
        println!("{username} does not exist");
        return;
    }
}

fn main() {
    let cli = Args::parse();
    match cli.command {
        Some(Commands::List) => {
            list_users();
        }
        Some(Commands::Add {
            username,
            password,
            admin,
        }) => {
            add_user(username, password, admin.unwrap_or(false));
        }
        Some(Commands::Delete { username }) => {
            delete_user(username);
        }
        Some(Commands::ChangePassword {
            username,
            new_password,
        }) => {
            change_password(username, new_password);
        }
        None => {
            println!("Run with --help to see more instructions!")
        }
    }
}
