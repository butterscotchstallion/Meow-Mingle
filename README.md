## Meow & Mingle

A cat dating app to find your purrfect match

## Screenshots

### Edit Profile

![Edit Profile](screenshots/edit-profile.png)

## Tech Stack

- Rust
- Axum
- Postgres
- Docker
- React
- Code generation using [utoipea](https://github.com/heyapi/utoipea) and [heyapi](https://github.com/heyapi/heyapi)

## Run the app

- Postgres: `docker compose up -d`
- API: `cargo run`
- Front end: `cd ui/src; pnpm run dev`

## Run tests

`cargo nextest run`

## Run test coverage report

`cargo llvm-cov nextest`

## Run password hashing utility

`cargo run --bin hash_password -- "yourpassword"`
