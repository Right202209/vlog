PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY,
    user_id INTEGER NOT NULL,
    csrf_token TEXT NOT NULL,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_sessions_user
    ON sessions(user_id);

CREATE INDEX IF NOT EXISTS idx_sessions_expires
    ON sessions(expires_at);

CREATE TABLE IF NOT EXISTS site_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS assets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    original_name TEXT NOT NULL,
    stored_path TEXT NOT NULL UNIQUE,
    mime TEXT NOT NULL,
    byte_size INTEGER NOT NULL,
    created_at TEXT NOT NULL
);

INSERT INTO site_settings (key, value, updated_at)
VALUES
    ('site_name',         'Volo Blog',                                            datetime('now')),
    ('site_subtitle',     '',                                                     datetime('now')),
    ('site_description',  'A lightweight Markdown blog powered by Volo-HTTP.',    datetime('now')),
    ('footer_copyright',  '© Volo Blog',                                          datetime('now')),
    ('about_content',     'About this blog. Edit me in Admin → Settings.',        datetime('now')),
    ('posts_per_page',    '10',                                                   datetime('now')),
    ('seo_title_template','{title} | {site_name}',                                datetime('now'))
ON CONFLICT(key) DO NOTHING;
