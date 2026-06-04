PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE COLLATE NOCASE,
    password_hash TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    modification_date INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);

CREATE TABLE IF NOT EXISTS sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    modification_date INTEGER NOT NULL,
    revoked_at INTEGER,
    last_seen_at INTEGER
);

CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_expires_at ON sessions(expires_at);

CREATE TABLE IF NOT EXISTS posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    pinned INTEGER NOT NULL DEFAULT 0,
    image TEXT,
    description TEXT NOT NULL,
    author_id INTEGER NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    author_name TEXT NOT NULL,
    start_date TEXT,
    end_date TEXT,
    creation_date INTEGER NOT NULL,
    modification_date INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_posts_creation_date ON posts(creation_date);
CREATE INDEX IF NOT EXISTS idx_posts_start_date ON posts(start_date);
CREATE INDEX IF NOT EXISTS idx_posts_author_id ON posts(author_id);
CREATE INDEX IF NOT EXISTS idx_posts_pinned ON posts(pinned);

CREATE TABLE IF NOT EXISTS reactions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    post_id INTEGER NOT NULL REFERENCES posts(id) ON DELETE CASCADE,
    reaction TEXT NOT NULL,
    UNIQUE(user_id, post_id)
);

CREATE INDEX IF NOT EXISTS idx_reactions_user_id ON reactions(user_id);
CREATE INDEX IF NOT EXISTS idx_reactions_post_id ON reactions(post_id);

CREATE TRIGGER IF NOT EXISTS trg_users_modification_date
AFTER UPDATE ON users
FOR EACH ROW
WHEN NEW.modification_date <= OLD.modification_date
BEGIN
    UPDATE users
    SET modification_date = unixepoch()
    WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_sessions_modification_date
AFTER UPDATE ON sessions
FOR EACH ROW
WHEN NEW.modification_date <= OLD.modification_date
BEGIN
    UPDATE sessions
    SET modification_date = unixepoch()
    WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS trg_posts_modification_date
AFTER UPDATE ON posts
FOR EACH ROW
WHEN NEW.modification_date <= OLD.modification_date
BEGIN
    UPDATE posts
    SET modification_date = unixepoch()
    WHERE id = NEW.id;
END;
