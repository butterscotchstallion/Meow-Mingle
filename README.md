## Meow & Mingle

A cat dating app to find your purrfect match

## Tech Stack

- Rust
- Axum
- Postgres
- Docker
- React
- Code generation using [utoipea](https://github.com/heyapi/utoipea) and [heyapi](https://github.com/heyapi/heyapi)

## Run the app

`docker compose up -d`

## Run tests

`cargo nextest run`

## Run password hashing utility

`cargo run --bin hash_password -- "yourpassword"`