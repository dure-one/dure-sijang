CREATE TABLE cached_products (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    store_url TEXT NOT NULL,
    product_id TEXT NOT NULL,
    product_data TEXT NOT NULL,
    cached_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(store_url, product_id)
);

CREATE INDEX idx_cached_products_store_url ON cached_products(store_url);
