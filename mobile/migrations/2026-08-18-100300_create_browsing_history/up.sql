CREATE TABLE browsing_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tab_id INTEGER NOT NULL,
    url TEXT NOT NULL,
    title TEXT,
    visited_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (tab_id) REFERENCES tabs(id) ON DELETE CASCADE
);

CREATE INDEX idx_browsing_history_visited_at ON browsing_history(visited_at DESC);
