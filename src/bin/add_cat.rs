use meow_mingle::handlers::auth::AuthSignUpPayload;
use meow_mingle::models::cat::NewCat;
use rand::distr::{Alphanumeric, SampleString};
use rand::Rng;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mut rng = rand::rng();

    if args.len() < 2 {
        eprintln!("Usage: add_cat <name> [age] [breed_id]");
        std::process::exit(1);
    }

    let name = &args[1];

    // Default age to random number between 6 and 30
    let age: i32 = args
        .get(2)
        .and_then(|a| a.parse().ok())
        .unwrap_or_else(|| rng.random_range(6..=30));

    // Default breed_id to Maine Coon
    let default_breed_id = "910ee31d-1fb6-428c-8b84-418cb8e55f20";

    // Generate a random password
    let password = Alphanumeric.sample_string(&mut rng, 16);

    let payload = AuthSignUpPayload {
        cat: NewCat {
            name: name.to_string(),
            password: password.clone(),
            age: Some(age),
            breed_id: default_breed_id.parse()?,
        },
    };

    let client = reqwest::Client::new();
    let res = client
        .post("http://127.0.0.1:3000/api/v1/auth/sign-up")
        .json(&payload)
        .send()
        .await?;

    if res.status().is_success() {
        println!("--------------------------------------------");
        println!("Successfully registered {name} ({}y old)!", age);
        println!("Password: {}", password);
        println!("{:#?}", res.json::<serde_json::Value>().await?);
        println!("--------------------------------------------");
    } else {
        eprintln!("Failed to register cat. Status: {}", res.status());
        eprintln!("{:#?}", res.text().await?);
    }

    Ok(())
}
