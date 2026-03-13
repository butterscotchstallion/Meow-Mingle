use crate::common::helpers::get_server;
use axum::http::StatusCode;
use meow_mingle::handlers::auth::routes::{AUTH_SIGN_IN, AUTH_SIGN_UP};
use meow_mingle::handlers::auth::{
    AuthResponseWithSessionInfo, AuthSignInPayload, AuthSignUpPayload, AuthSignUpResponse,
};
use meow_mingle::models::cat::NewCat;
use meow_mingle::models::status::Status;
use names::Generator;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use tracing::log::info;
use uuid::Uuid;

static SESSION_CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

fn session_cache() -> &'static Mutex<HashMap<String, String>> {
    SESSION_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

// Get a session id and verify it's valid
// Tracks the user session id so this should work
// with multiple users
#[allow(dead_code)]
pub async fn get_session_id_and_verify(name: String, password: String) -> String {
    if let Some(id) = session_cache().lock().unwrap().get(&name).cloned() {
        return id;
    }

    let server = get_server().await;
    let cat_name = name.clone();
    let response = server
        .post(AUTH_SIGN_IN)
        .json(&AuthSignInPayload {
            name: name.clone(),
            password,
        })
        .await;
    response.assert_status(StatusCode::OK);

    let body = response.json::<AuthResponseWithSessionInfo>();
    let results = &body.results;

    let session_id = response
        .cookie(meow_mingle::models::session::SESSION_COOKIE_NAME)
        .value()
        .parse::<Uuid>()
        .expect("session cookie should be a valid UUID");

    assert_eq!(results.session_id.len(), 36);
    assert_eq!(session_id, results.session_id.parse::<Uuid>().unwrap());
    assert_eq!(body.status, "OK");
    assert_eq!(body.message, "Sign in successful");
    assert_eq!(results.cat.name, cat_name);
    assert_eq!(results.cat.password, "");

    let session_id_str = body.results.session_id.clone();
    session_cache()
        .lock()
        .unwrap()
        .insert(name, session_id_str.clone());

    session_id_str
}

// Should be used when fetching a session id in tests because
// they run in parallel, and there is a unique index on cat id,
// so every test that runs is replacing the session id in the DB.
// Using separate users for each test solves this issue.
// Returns session id after signing up
#[allow(dead_code)]
pub async fn sign_up_and_get_session_id() -> String {
    let server = get_server().await;
    let mut generator = Generator::default();
    let name = generator.next().expect("Failed to generate a name");
    let password = Uuid::new_v4().to_string();
    let maine_coon_breed_id = "910ee31d-1fb6-428c-8b84-418cb8e55f20";
    let age = Option::from(3);
    let cat = NewCat {
        name: name.clone(),
        password: password.clone(),
        age,
        breed_id: maine_coon_breed_id.parse().unwrap(),
    };

    info!("signing up with cat: {:?}", cat);

    let response = server
        .post(AUTH_SIGN_UP)
        .json(&AuthSignUpPayload { cat })
        .await;

    response.assert_status(StatusCode::CREATED);

    let body = response.json::<AuthSignUpResponse>();
    let results = body.results.as_ref().expect("results should be present");

    assert_eq!(body.status, Status::Ok);
    assert_eq!(results.cat.name, name);
    assert_eq!(results.cat.age, age);
    assert_eq!(
        results.cat.breed_id.unwrap().to_string(),
        maine_coon_breed_id
    );
    assert_eq!(results.cat.password, "");

    results
        .session_id
        .parse::<Uuid>()
        .expect("session id should be valid UUID")
        .to_string()
}
