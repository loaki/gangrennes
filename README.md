# gangrennes

Minimal, safe, and cleanly structured Axum API with SQLite.

## Structure

```
src/
  main.rs
  lib.rs
  config.rs
  db/
  models/
  routes/
  utils/
  controllers/
migrations/
```

## Endpoints

- `GET /docs`
- `GET /api-doc/openapi.json`
- `GET /health`
- `POST /auth/register`
- `POST /auth/login`
- `GET /api/items`
- `GET /api/items/{id}`
- `POST /api/items`
- `DELETE /api/items/{id}`

`POST /auth/register` body:

```json
{
  "name": "alice",
  "password": "strong-password-with-12-or-more-chars"
}
```

`POST /auth/login` body:

```json
{
  "name": "alice",
  "password": "strong-password-with-12-or-more-chars"
}
```

Both auth endpoints return:

```json
{
  "token": "jwt-token",
  "user": {
    "id": "uuid",
    "name": "alice",
    "creation_date": "...",
    "modification_date": "..."
  }
}
```

`POST /api/items` body:

```json
{
  "name": "example",
  "description": "optional"
}
```

## Local Run

```bash
cargo run
```

Open API documentation:

- http://127.0.0.1:3000/docs
- http://127.0.0.1:3000/api-doc/openapi.json

## Docker Compose Run

```bash
docker compose up --build
```