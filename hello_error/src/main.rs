use serde::Deserialize;
use std::{io::read_to_string, path::Path};
use thiserror::Error;

#[derive(Debug, Error)]
enum UsersError {
    #[error("no users found")]
    NoUsers,
    #[error("too many users found")]
    TooManyUsers,
}

#[derive(Deserialize, Debug)]
struct User {
    user: String,
}

fn main() {
    simple_error();
    error_match();
    let _ = maybe_read_a_file();

    if let Ok(content) = file_to_uppercase() {
        println!("contents: {content}");
    }

    let result = load_users_generic();
    match result {
        Ok(users) => {
            println!("{users:?}")
        }
        Err(e) => {
            println!("error: {e}")
        }
    }

    let result = load_users();
    match result {
        Ok(users) => {
            println!("{users:?}")
        }
        Err(e) => println!("{e}"),
    }
}

fn simple_error() {
    let my_file = Path::new("myfile.txt");
    let content = std::fs::read_to_string(my_file);
    match content {
        Ok(contents) => {
            println!("{contents}");
        }
        Err(e) => {
            println!("Error: {e:#?}");
        }
    }
}
fn error_match() {
    let my_file = Path::new("myfile.txt");
    let content = std::fs::read_to_string(my_file);
    match content {
        Ok(contents) => {
            println!("{contents}");
        }
        Err(e) => match e.kind() {
            std::io::ErrorKind::NotFound => println!("file not found - myfile.txt"),
            _ => println!("Error: {e:#?}"),
        },
    }
}

fn maybe_read_a_file() -> Result<String, std::io::Error> {
    let my_file = Path::new("myfile.txt");
    return std::fs::read_to_string(my_file);
}

fn file_to_uppercase() -> Result<String, std::io::Error> {
    let my_file = Path::new("myfile.txt");
    let result = std::fs::read_to_string(my_file)?;
    return Ok(result.to_uppercase());
}

type GenericResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn load_users_generic() -> GenericResult<Vec<User>> {
    let my_path = Path::new("users.json");
    let raw_text = std::fs::read_to_string(my_path)?;
    let users: Vec<User> = serde_json::from_str(&raw_text)?;
    return Ok(users);
}

fn load_users() -> Result<Vec<User>, UsersError> {
    let my_path = Path::new("users.json");
    let raw_text = std::fs::read_to_string(my_path).map_err(|_| UsersError::NoUsers)?;
    let users: Vec<User> = serde_json::from_str(&raw_text).map_err(|_| UsersError::NoUsers)?;
    return Ok(users);
}
