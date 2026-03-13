use axum::http::StatusCode;
use futures::future::join_all;
use meow_mingle::handlers::auth::routes::AUTH_SIGN_UP;
use meow_mingle::handlers::auth::{AuthSignUpPayload, AuthSignUpResponse};
use meow_mingle::models::cat::NewCat;
use rand::RngExt;
use time::OffsetDateTime;
use uuid::Uuid;

#[allow(dead_code)]
mod common;
use common::helpers::get_server;

#[tokio::test]
async fn test_parallel_registration_logic() {
    let server = get_server().await;
    let names = vec![Uuid::new_v4().to_string(), Uuid::new_v4().to_string()];
    let mut rng = rand::rng();
    let years_ago: i64 = rng.random_range(6..=20);
    let birth_date = OffsetDateTime::now_utc() - time::Duration::days(years_ago * 365);

    // Use join_all instead of tokio::spawn to avoid Send + Clone requirements on TestServer
    let tasks: Vec<_> = names
        .into_iter()
        .map(|name| async move {
            let cat = NewCat {
                name: name.clone(),
                password: "password123".to_string(),
                birth_date: Some(birth_date),
                breed_id: "910ee31d-1fb6-428c-8b84-418cb8e55f20".parse().unwrap(),
            };
            server
                .post(AUTH_SIGN_UP)
                .json(&AuthSignUpPayload { cat })
                .await
        })
        .collect();

    let results = join_all(tasks).await;

    for response in results {
        response.assert_status(StatusCode::CREATED);
        let body = response.json::<AuthSignUpResponse>();
        assert!(body.results.is_some(), "Expected results in response body");
    }
}
