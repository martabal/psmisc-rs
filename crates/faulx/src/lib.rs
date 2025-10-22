use std::process;

use nix::unistd::{User, setuid};

pub mod cli;
pub mod macros;
pub mod processes;
pub mod signals;

pub fn change_user(username: &str) {
    let user = match User::from_name(username) {
        Ok(Some(user)) => user,
        Ok(None) => {
            eprintln!("Cannot find user {username}");
            process::exit(1)
        }
        Err(e) => {
            eprintln!("Error looking up user {username}: {e}");
            process::exit(1);
        }
    };

    if let Err(e) = setuid(user.uid) {
        eprintln!("Error changing to user {username}: {e}");
        process::exit(1);
    }
}
