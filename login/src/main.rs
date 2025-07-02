use authentication::{login, read_line, LoginAction, LoginRole};

fn main() {
    let mut tries: i32 = 0;
    loop {
        println!("Enter your username");
        let username: String = read_line();
        println!("Enter your password");
        let password: String = read_line();

        match login(&username, &password) {
            Some(LoginAction::Granted(LoginRole::Admin)) => {
                println!("Welcome {username}, you are admin");
                break;
            }
            Some(LoginAction::Granted(LoginRole::User)) => {
                println!("Welcome {username}, you are user");
                break;
            }
            Some(LoginAction::Denied) => {
                println!("Incorrect username or password");
                tries += 1;
                if tries >= 3 {
                    println!("Too many failed logins");
                    break;
                }
            }
            None => {
                println!("New user system");
            }
        }
    }
}
