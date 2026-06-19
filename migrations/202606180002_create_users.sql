CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE CHECK (length(trim(name)) > 0 AND length(name) <= 80),
    password_hash TEXT NOT NULL,
    creation_date TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
    modification_date TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
);

CREATE TRIGGER IF NOT EXISTS trg_users_modification_date
AFTER UPDATE ON users
FOR EACH ROW
WHEN NEW.modification_date = OLD.modification_date
BEGIN
    UPDATE users
    SET modification_date = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
    WHERE id = OLD.id;
END;