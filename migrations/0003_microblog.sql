PRAGMA foreign_keys = ON;

-- ===== users: profile + role columns =====
-- Existing rows are admin bootstrap accounts; backfill role to 'admin'.
-- Future accounts default to 'user' and get created from /admin/users.

ALTER TABLE users ADD COLUMN display_name TEXT;
ALTER TABLE users ADD COLUMN bio TEXT;
ALTER TABLE users ADD COLUMN avatar_url TEXT;
ALTER TABLE users ADD COLUMN role TEXT NOT NULL DEFAULT 'user'
    CHECK (role IN ('user', 'admin'));

UPDATE users
SET role = 'admin'
WHERE role IS NULL OR role = '' OR role = 'user';

UPDATE users
SET display_name = username
WHERE display_name IS NULL OR display_name = '';

-- ===== statuses (microblog) =====

CREATE TABLE IF NOT EXISTS statuses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    content_md TEXT NOT NULL,
    content_html TEXT NOT NULL,
    parent_id INTEGER,
    repost_of_id INTEGER,
    reply_count INTEGER NOT NULL DEFAULT 0,
    like_count INTEGER NOT NULL DEFAULT 0,
    repost_count INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES statuses(id) ON DELETE CASCADE,
    FOREIGN KEY (repost_of_id) REFERENCES statuses(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_statuses_recent
    ON statuses(created_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_statuses_user_recent
    ON statuses(user_id, created_at DESC, id DESC);

CREATE INDEX IF NOT EXISTS idx_statuses_parent
    ON statuses(parent_id);

CREATE INDEX IF NOT EXISTS idx_statuses_repost_of
    ON statuses(repost_of_id);

-- ===== status_assets =====

CREATE TABLE IF NOT EXISTS status_assets (
    status_id INTEGER NOT NULL,
    asset_id INTEGER NOT NULL,
    sort INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (status_id, asset_id),
    FOREIGN KEY (status_id) REFERENCES statuses(id) ON DELETE CASCADE,
    FOREIGN KEY (asset_id) REFERENCES assets(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_status_assets_status
    ON status_assets(status_id, sort);

-- ===== likes =====

CREATE TABLE IF NOT EXISTS likes (
    user_id INTEGER NOT NULL,
    status_id INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (user_id, status_id),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (status_id) REFERENCES statuses(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_likes_status
    ON likes(status_id);

-- ===== follows =====

CREATE TABLE IF NOT EXISTS follows (
    follower_id INTEGER NOT NULL,
    followee_id INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (follower_id, followee_id),
    CHECK (follower_id <> followee_id),
    FOREIGN KEY (follower_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (followee_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_follows_followee
    ON follows(followee_id);

-- ===== count-maintenance triggers =====
-- Keep statuses.reply_count / repost_count / like_count current.
-- UPDATE on a non-existent row is a no-op, so cascades are safe.

CREATE TRIGGER IF NOT EXISTS trg_statuses_after_insert_reply
AFTER INSERT ON statuses
WHEN NEW.parent_id IS NOT NULL
BEGIN
    UPDATE statuses
    SET reply_count = reply_count + 1
    WHERE id = NEW.parent_id;
END;

CREATE TRIGGER IF NOT EXISTS trg_statuses_after_delete_reply
AFTER DELETE ON statuses
WHEN OLD.parent_id IS NOT NULL
BEGIN
    UPDATE statuses
    SET reply_count = MAX(reply_count - 1, 0)
    WHERE id = OLD.parent_id;
END;

CREATE TRIGGER IF NOT EXISTS trg_statuses_after_insert_repost
AFTER INSERT ON statuses
WHEN NEW.repost_of_id IS NOT NULL
BEGIN
    UPDATE statuses
    SET repost_count = repost_count + 1
    WHERE id = NEW.repost_of_id;
END;

CREATE TRIGGER IF NOT EXISTS trg_statuses_after_delete_repost
AFTER DELETE ON statuses
WHEN OLD.repost_of_id IS NOT NULL
BEGIN
    UPDATE statuses
    SET repost_count = MAX(repost_count - 1, 0)
    WHERE id = OLD.repost_of_id;
END;

CREATE TRIGGER IF NOT EXISTS trg_likes_after_insert
AFTER INSERT ON likes
BEGIN
    UPDATE statuses
    SET like_count = like_count + 1
    WHERE id = NEW.status_id;
END;

CREATE TRIGGER IF NOT EXISTS trg_likes_after_delete
AFTER DELETE ON likes
BEGIN
    UPDATE statuses
    SET like_count = MAX(like_count - 1, 0)
    WHERE id = OLD.status_id;
END;
