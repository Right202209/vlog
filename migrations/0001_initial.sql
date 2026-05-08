PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS posts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    summary TEXT,
    content_md TEXT NOT NULL,
    content_html TEXT NOT NULL,
    cover_image TEXT,
    status TEXT NOT NULL CHECK (status IN ('draft', 'published', 'archived')),
    category_id INTEGER,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    published_at TEXT,
    FOREIGN KEY (category_id) REFERENCES categories(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_posts_status_published
    ON posts(status, published_at DESC);

CREATE TABLE IF NOT EXISTS post_tags (
    post_id INTEGER NOT NULL,
    tag_id INTEGER NOT NULL,
    PRIMARY KEY (post_id, tag_id),
    FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE
);

INSERT INTO categories (name, slug, description)
VALUES
    ('Tech', 'tech', 'Technical articles'),
    ('Notes', 'notes', 'Short field notes')
ON CONFLICT(slug) DO NOTHING;

INSERT INTO tags (name, slug)
VALUES
    ('Rust', 'rust'),
    ('Volo', 'volo'),
    ('SQLite', 'sqlite')
ON CONFLICT(slug) DO NOTHING;

INSERT INTO posts (
    title, slug, summary, content_md, content_html, cover_image, status,
    category_id, created_at, updated_at, published_at
)
SELECT
    'Hello World',
    'hello-world',
    'First post from the seeded Volo Blog.',
    '# Hello World

This is the first post rendered from SQLite.',
    '<h1>Hello World</h1><p>This is the first post rendered from SQLite.</p>',
    NULL,
    'published',
    categories.id,
    datetime('now', '-2 days'),
    datetime('now', '-2 days'),
    datetime('now', '-2 days')
FROM categories
WHERE categories.slug = 'tech'
ON CONFLICT(slug) DO NOTHING;

INSERT INTO posts (
    title, slug, summary, content_md, content_html, cover_image, status,
    category_id, created_at, updated_at, published_at
)
SELECT
    'Building Small Rust Services',
    'building-small-rust-services',
    'A short note on keeping Rust web services compact.',
    '## Keep it small

Start with simple routes, embedded migrations, and server-rendered pages.',
    '<h2>Keep it small</h2><p>Start with simple routes, embedded migrations, and server-rendered pages.</p>',
    NULL,
    'published',
    categories.id,
    datetime('now', '-1 day'),
    datetime('now', '-1 day'),
    datetime('now', '-1 day')
FROM categories
WHERE categories.slug = 'notes'
ON CONFLICT(slug) DO NOTHING;

INSERT OR IGNORE INTO post_tags (post_id, tag_id)
SELECT posts.id, tags.id
FROM posts, tags
WHERE posts.slug = 'hello-world' AND tags.slug IN ('rust', 'volo');

INSERT OR IGNORE INTO post_tags (post_id, tag_id)
SELECT posts.id, tags.id
FROM posts, tags
WHERE posts.slug = 'building-small-rust-services' AND tags.slug IN ('rust', 'sqlite');

