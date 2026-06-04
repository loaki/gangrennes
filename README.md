# La Gangrennes

Rust backend for a small blog stack with SQLite and Docker Compose.

## Architecture

Minimal and feature-oriented layout:

```text
src/
	domain/
		auth.rs
		posts.rs
	http/
		handlers.rs
		router.rs
		views.rs
	infra/
		db.rs
	config.rs
	error.rs
	state.rs
	lib.rs
	main.rs

migrations/
	0001_init.sql
	0002_add_modification_date_and_triggers.sql
```

## What is in place

- Axum HTTP server with server-side sessions.
- SQLite storage with WAL and foreign keys enabled.
- SQL migrations loaded from `migrations/*.sql` at startup.
- Argon2 password hashing.
- Minimal HTML templates for `/`, `/login`, `/new`, `/pinned`, `/calendar`, and `/profile`.
- Docker Compose and a multi-stage Dockerfile.

## Run locally

```bash
cargo run
```

On startup, migrations are applied automatically.

## Run with Docker Compose

```bash
docker compose up --build
```

## Migration workflow

Install CLI once:

```bash
cargo install sqlx-cli --no-default-features --features sqlite
```

Create a migration:

```bash
sqlx migrate add add_profile_table
```

Run migrations manually (optional because app startup also runs them):

```bash
sqlx migrate run --database-url sqlite://data/gangrennes.sqlite3
```

Revert last migration:

```bash
sqlx migrate revert --database-url sqlite://data/gangrennes.sqlite3
```

### Example: `modification_date` + triggers

See `migrations/0002_add_modification_date_and_triggers.sql` for:

- adding `modification_date` to `users` and `sessions`
- backfilling from `created_at`
- trigger-based automatic update on row modification
- same trigger behavior for `posts`

## Security notes

- Passwords are never stored in plaintext.
- Session cookies are `HttpOnly` and `SameSite=Lax`.
- SQLite is configured with WAL, foreign keys, and a busy timeout.
- Login and register bodies are capped to a small size.
- Set `COOKIE_SECURE=true` when serving behind HTTPS.
