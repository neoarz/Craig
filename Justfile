set dotenv-load

default: run

# Cargo
run:
    cargo run

check:
    cargo fmt
    cargo clippy -- -D warnings
    cargo check

build:
    cargo build --release

clean:
    cargo clean

update:
    cargo update

# Database
db:
    pgcli "$DATABASE_URL"

migrate name:
    sqlx migrate add {{name}}

migrate-run:
    sqlx migrate run

migrate-revert:
    sqlx migrate revert
