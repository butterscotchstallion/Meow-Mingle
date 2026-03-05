use meow_mingle::hasher;
use std::env;

fn main() {
    let password = env::args().nth(1).expect("Usage: hash_password <password>");
    match hasher::hash_password(&password) {
        Ok(hash) => println!("{}", hash),
        Err(e) => eprintln!("Error hashing password: {}", e),
    }
}
