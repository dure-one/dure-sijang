CREATE TABLE cached_carts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    store_url TEXT NOT NULL,
    cart_id TEXT NOT NULL,
    cart_data TEXT NOT NULL,
    cached_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(store_url, cart_id)
);

CREATE INDEX idx_cached_carts_store_url ON cached_carts(store_url);
