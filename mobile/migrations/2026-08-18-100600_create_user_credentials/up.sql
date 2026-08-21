CREATE TABLE user_credentials (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    store_url TEXT NOT NULL UNIQUE,
    username TEXT,
    encrypted_password TEXT,
    auth_token TEXT,
    token_expires_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
