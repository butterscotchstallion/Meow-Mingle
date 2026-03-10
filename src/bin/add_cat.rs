use futures::stream::{FuturesUnordered, StreamExt};
use meow_mingle::handlers::auth::AuthSignUpPayload;
use meow_mingle::models::cat::NewCat;
use rand::distr::{Alphanumeric, SampleString};
use rand::Rng;
use std::error::Error;
use std::fs;

type BoxError = Box<dyn Error>;

fn read_file_to_vec(path: &str) -> Result<Vec<String>, BoxError> {
    let contents = fs::read_to_string(path)?;
    Ok(contents.split_whitespace().map(|s| s.to_string()).collect())
}

async fn add_cat_request(name: String) -> Result<(), BoxError> {
    let mut rng = rand::rng();
    let age: i32 = rng.random_range(6..=30);
    let default_breed_id = "910ee31d-1fb6-428c-8b84-418cb8e55f20";
    let password = Alphanumeric.sample_string(&mut rng, 16);

    let payload = AuthSignUpPayload {
        cat: NewCat {
            name: name.clone(),
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
        println!("Successfully registered {name}!");
        println!("Password: {}", password.clone());
        Ok(())
    } else {
        let status = res.status();
        let body = res.text().await.unwrap_or_default();
        Err(format!("Failed to register {name} ({}): {}", status, body).into())
    }
}

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    let cats = read_file_to_vec("data/cat_names.txt")?;

    let mut futures: FuturesUnordered<_> =
        cats.into_iter().map(|cat| add_cat_request(cat)).collect();

    while let Some(result) = futures.next().await {
        result?;
    }

    Ok(())
}
