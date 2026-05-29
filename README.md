# Minimal Actix Editable Board

A small minimalist website built with Rust + Actix, containerized with Docker Compose.

## Features

- Blank minimalist board
- Edit toggle button
- Editing tools:
  - Move elements
  - Add text elements
  - Draw freehand strokes
- All created elements are movable
- Responsive layout for mobile and desktop

## Run with Docker Compose

```bash
docker compose up --build
```

Open: http://localhost:8080

## Faster Iteration

### 1) Faster rebuilds (default compose)

The Dockerfile is cache-optimized so changes under `static/` do not trigger a Rust recompile.

```bash
docker compose up -d --build
```

### 2) Instant frontend changes (no rebuild)

Use the dev override to bind-mount `static/` into the container.

```bash
docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d --build
```

After that, edits in `static/` are visible on refresh without rebuilding.

## Controls

- Click **Edit** to enter edit mode.
- Use **Move** to drag elements by their handle.
- Use **Text** then tap/click the board to add editable text.
- Use **Draw** then drag on empty board space to create a drawing.
- Use **Clear** to remove all elements while in edit mode.
